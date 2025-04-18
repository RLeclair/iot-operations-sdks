// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.
package protocol

import (
	"context"
	"errors"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"slices"
	"strconv"
	"strings"
	"sync"
	"testing"

	"github.com/Azure/iot-operations-sdks/go/protocol"
	"github.com/BurntSushi/toml"
	"github.com/eclipse/paho.golang/packets"
	"github.com/google/uuid"
	"github.com/stretchr/testify/require"
	"gopkg.in/yaml.v3"
)

func RunCommandExecutorTests(t *testing.T) {
	var commandExecutorDefaultInfo DefaultTestCase

	_, err := toml.DecodeFile(
		"../../../eng/test/test-cases/Protocol/CommandExecutor/defaults.toml",
		&commandExecutorDefaultInfo,
	)
	if err != nil {
		panic(err)
	}

	TestCaseDefaultInfo = &commandExecutorDefaultInfo
	TestCaseDefaultSerializer = &commandExecutorDefaultInfo.Prologue.Executor.Serializer

	files, err := filepath.Glob(
		"../../../eng/test/test-cases/Protocol/CommandExecutor/*.yaml",
	)
	if err != nil {
		log.Fatal(err)
	}

	enableFreezing()

	for ix, f := range files {
		testName, _ := strings.CutSuffix(filepath.Base(f), ".yaml")
		t.Run(testName, func(t *testing.T) {
			runOneCommandExecutorTest(t, ix, testName, f)
		})
	}
}

func runOneCommandExecutorTest(
	t *testing.T,
	testCaseIndex int,
	testName string,
	fileName string,
) {
	pendingTestCases := []string{}

	testCaseYaml, err := os.ReadFile(fileName)
	if err != nil {
		log.Fatal(err)
	}

	//	t.Parallel()

	var testCase TestCase
	err = yaml.Unmarshal(testCaseYaml, &testCase)
	if err != nil {
		log.Fatal(err)
	}

	if slices.Contains(testCase.Requires, Unobtanium) ||
		slices.Contains(testCase.Requires, ExplicitDefault) {
		t.Skipf(
			"Skipping test %s because it requires an unavailable feature",
			testName,
		)
	}

	if slices.Contains(pendingTestCases, testName) {
		t.Skipf(
			"Skipping test %s because it requires a feature which has not yet been implemented",
			testName,
		)
	}

	var mqttClientID string
	if testCase.Prologue.MqttConfig.ClientID != nil {
		mqttClientID = *testCase.Prologue.MqttConfig.ClientID
	} else {
		mqttClientID = fmt.Sprintf("ExecutorTestClient%d", testCaseIndex)
	}

	countdownEvents := make(map[string]*CountdownEvent)
	for name, init := range testCase.Prologue.CountdownEvents {
		countdownEvents[name] = NewCountdownEvent(init)
	}

	stubBroker, sessionClient := getStubAndSessionClient(t, mqttClientID)

	for _, ackKind := range testCase.Prologue.PushAcks.Publish {
		stubBroker.EnqueuePubAck(ackKind)
	}

	for _, ackKind := range testCase.Prologue.PushAcks.Subscribe {
		stubBroker.EnqueueSubAck(ackKind)
	}

	for _, ackKind := range testCase.Prologue.PushAcks.Unsubscribe {
		stubBroker.EnqueueUnsubAck(ackKind)
	}

	var commandExecutors []*TestingCommandExecutor

	for ix := range testCase.Prologue.Executors {
		var catch *TestCaseCatch
		if ix == len(testCase.Prologue.Executors)-1 {
			catch = testCase.Prologue.Catch
		}

		executor := getCommandExecutor(
			t,
			sessionClient,
			&testCase.Prologue.Executors[ix],
			countdownEvents,
			catch,
		)
		if executor != nil {
			commandExecutors = append(commandExecutors, executor)
		}
	}

	sourceIDs := make(map[int]string)
	correlationIDs := make(map[int][]byte)
	packetIDs := make(map[int]uint16)

	freezeTicket := -1
	defer func() {
		if freezeTicket >= 0 {
			unfreezeTime(freezeTicket)
		}
	}()

	for _, action := range testCase.Actions {
		switch action.Kind {
		case ReceiveRequest:
			receiveRequest(
				action.AsReceiveRequest(),
				stubBroker,
				sourceIDs,
				correlationIDs,
				packetIDs,
			)
		case AwaitAck:
			awaitAcknowledgement(t, action.AsAwaitAck(), stubBroker, packetIDs)
		case AwaitPublish:
			awaitPublishResponse(
				t,
				action.AsAwaitPublish(),
				stubBroker,
				correlationIDs,
			)
		case Sync:
			syncEvent(t, action.AsSync(), countdownEvents)
		case Sleep:
			sleep(action.AsSleep())
		case Disconnect:
			stubBroker.Disconnect()
		case FreezeTime:
			freezeTicket = freezeTime()
		case UnfreezeTime:
			unfreezeTime(freezeTicket)
			freezeTicket = -1
		}
	}

	for _, topic := range testCase.Epilogue.SubscribedTopics {
		require.True(t, stubBroker.HasSubscribed(topic))
	}

	if testCase.Epilogue.PublicationCount != nil {
		require.Equal(
			t,
			*testCase.Epilogue.PublicationCount,
			stubBroker.PublicationCount,
		)
	}

	for _, publishedMessage := range testCase.Epilogue.PublishedMessages {
		checkPublishedResponse(t, &publishedMessage, stubBroker, correlationIDs)
	}

	if testCase.Epilogue.AcknowledgementCount != nil {
		require.Equal(
			t,
			*testCase.Epilogue.AcknowledgementCount,
			stubBroker.AcknowledgementCount,
		)
	}

	if testCase.Epilogue.ExecutionCount != nil {
		require.Equal(
			t,
			*testCase.Epilogue.ExecutionCount,
			commandExecutors[0].executionCount,
		)
	}

	for ix, cmdExecutor := range commandExecutors {
		if exeCount, ok := testCase.Epilogue.ExecutionCounts[ix]; ok {
			require.Equal(t, exeCount, cmdExecutor.executionCount)
		}
	}
}

func getCommandExecutor(
	t *testing.T,
	sessionClient protocol.MqttClient,
	tce *TestCaseExecutor,
	countdownEvents map[string]*CountdownEvent,
	catch *TestCaseCatch,
) *TestingCommandExecutor {
	options := []protocol.CommandExecutorOption{
		protocol.WithTopicTokens(tce.TopicTokenMap),
		protocol.WithIdempotent(tce.Idempotent),
		protocol.WithTimeout(tce.ExecutionTimeout.ToDuration()),
	}

	if tce.TopicNamespace != nil {
		options = append(
			options,
			protocol.WithTopicNamespace(*tce.TopicNamespace),
		)
	}

	if tce.ExecutionConcurrency != nil {
		options = append(
			options,
			protocol.WithConcurrency(*tce.ExecutionConcurrency),
		)
	}

	executor, err := NewTestingCommandExecutor(
		sessionClient,
		tce.CommandName,
		&tce.Serializer,
		tce.RequestTopic,
		func(
			ctx context.Context,
			req *protocol.CommandRequest[string],
			reqRespSeq *sync.Map,
		) (*protocol.CommandResponse[string], error) {
			return processRequest(ctx, req, tce, countdownEvents, reqRespSeq)
		},
		options...)

	if err == nil {
		err = executor.base.Start(context.Background())
	}

	if catch == nil {
		require.NoErrorf(
			t,
			err,
			"Unexpected error returned when initializing CommandExecutor: %s",
			err,
		)
	} else {
		require.Errorf(t, err, "Expected %s error, but no error returned when initializing CommandExecutor", catch.ErrorKind)
		CheckError(t, *catch, err)
	}

	return executor
}

func receiveRequest(
	actionReceiveRequest *TestCaseActionReceiveRequest,
	stubBroker *StubBroker,
	sourceIDs map[int]string,
	correlationIDs map[int][]byte,
	packetIDs map[int]uint16,
) {
	var props packets.Properties

	if actionReceiveRequest.SourceIndex != nil {
		sourceID, ok := sourceIDs[*actionReceiveRequest.SourceIndex]
		if !ok {
			guid, _ := uuid.NewV7()
			sourceID = guid.String()
			sourceIDs[*actionReceiveRequest.SourceIndex] = sourceID
		}
		props.User = append(props.User, packets.User{
			Key:   SourceID,
			Value: sourceID,
		})
	}

	if actionReceiveRequest.CorrelationIndex != nil {
		var ok bool
		props.CorrelationData, ok = correlationIDs[*actionReceiveRequest.CorrelationIndex]
		if !ok {
			if actionReceiveRequest.CorrelationID != nil {
				props.CorrelationData = []byte(
					*actionReceiveRequest.CorrelationID,
				)
			} else {
				guid, _ := uuid.NewV7()
				props.CorrelationData = guid[:]
			}
			correlationIDs[*actionReceiveRequest.CorrelationIndex] = props.CorrelationData
		}
	}

	var packetID uint16
	if actionReceiveRequest.PacketIndex != nil {
		var ok bool
		packetID, ok = packetIDs[*actionReceiveRequest.PacketIndex]
		if !ok {
			packetID = stubBroker.GetNewPacketID()
		}
	} else {
		packetID = stubBroker.GetNewPacketID()
	}

	if actionReceiveRequest.ContentType != nil {
		props.ContentType = *actionReceiveRequest.ContentType
	}

	if actionReceiveRequest.FormatIndicator != nil {
		payloadFormat := byte(*actionReceiveRequest.FormatIndicator)
		props.PayloadFormat = &payloadFormat
	}

	if actionReceiveRequest.ResponseTopic != nil {
		props.ResponseTopic = *actionReceiveRequest.ResponseTopic
	}

	var payload []byte
	if actionReceiveRequest.Payload != nil {
		payload = []byte(*actionReceiveRequest.Payload)
	}

	if actionReceiveRequest.MessageExpiry != nil {
		messageExpiry := uint32(
			actionReceiveRequest.MessageExpiry.ToDuration().Seconds(),
		)
		props.MessageExpiry = &messageExpiry
	}

	for key, val := range actionReceiveRequest.Metadata {
		props.User = append(props.User, packets.User{
			Key:   key,
			Value: val,
		})
	}

	request := packets.Publish{
		PacketID:   packetID,
		Topic:      *actionReceiveRequest.Topic,
		Properties: &props,
		Payload:    payload,
	}

	if actionReceiveRequest.Qos != nil {
		request.QoS = byte(*actionReceiveRequest.Qos)
	}

	stubBroker.ReceiveMessage(&request)

	if actionReceiveRequest.PacketIndex != nil {
		packetIDs[*actionReceiveRequest.PacketIndex] = packetID
	}
}

func awaitPublishResponse(
	t *testing.T,
	actionAwaitPublish *TestCaseActionAwaitPublish,
	stubBroker *StubBroker,
	correlationIDs map[int][]byte,
) {
	correlationID := stubBroker.AwaitPublish()

	if actionAwaitPublish.CorrelationIndex != nil {
		extantCorrelationID, ok := correlationIDs[*actionAwaitPublish.CorrelationIndex]
		require.True(
			t,
			ok,
			"CorrelationIndex %d not found",
			*actionAwaitPublish.CorrelationIndex,
		)
		require.Equal(t, extantCorrelationID, correlationID)
	}
}

func syncEvent(
	t *testing.T,
	actionSync *TestCaseActionSync,
	countdownEvents map[string]*CountdownEvent,
) {
	if actionSync.WaitEvent != nil {
		err := countdownEvents[*actionSync.WaitEvent].Wait(context.Background())
		require.NoError(t, err)
	}

	if actionSync.SignalEvent != nil {
		countdownEvents[*actionSync.SignalEvent].Signal()
	}
}

func checkPublishedResponse(
	t *testing.T,
	publishedMessage *TestCasePublishedMessage,
	stubBroker *StubBroker,
	correlationIDs map[int][]byte,
) {
	var lookupKey []byte
	if publishedMessage.CorrelationIndex != nil {
		lookupKey = correlationIDs[*publishedMessage.CorrelationIndex]
	}

	msg, ok := stubBroker.GetPublishedMessage(lookupKey)
	require.True(t, ok)

	if publishedMessage.Topic != nil {
		require.Equal(t, *publishedMessage.Topic, msg.Topic)
	}

	if publishedMessage.Payload == nil {
		require.Empty(t, msg.Payload)
	} else if payload, ok := publishedMessage.Payload.(string); ok {
		require.Equal(t, payload, string(msg.Payload))
	}

	if publishedMessage.ContentType != nil {
		require.Equal(
			t,
			*publishedMessage.ContentType,
			msg.Properties.ContentType,
		)
	}

	if publishedMessage.FormatIndicator != nil {
		require.Equal(
			t,
			*publishedMessage.FormatIndicator,
			*msg.Properties.PayloadFormat,
		)
	}

	for key, val := range publishedMessage.Metadata {
		propVal, ok := getUserProperty(t, msg, key)
		if val != nil {
			require.True(t, ok)
			require.Equal(t, *val, propVal)
		} else {
			require.False(t, ok)
		}
	}

	if publishedMessage.CommandStatus == nil {
		_, ok := getUserProperty(t, msg, Status)
		require.False(t, ok)
	} else if cmdStatus, ok := publishedMessage.CommandStatus.(int); ok {
		status, ok := getUserProperty(t, msg, Status)
		require.True(t, ok)
		require.Equal(t, strconv.Itoa(cmdStatus), status)
	}

	if publishedMessage.IsApplicationError != nil {
		isAppErr, ok := getUserProperty(t, msg, IsApplicationError)
		if *publishedMessage.IsApplicationError {
			require.True(t, ok && strings.EqualFold(isAppErr, "true"))
		} else {
			require.True(t, !ok || strings.EqualFold(isAppErr, "false"))
		}
	}

	if publishedMessage.Expiry != nil {
		require.Equal(
			t,
			*publishedMessage.Expiry,
			*msg.Properties.MessageExpiry,
		)
	}
}

func processRequest(
	ctx context.Context,
	req *protocol.CommandRequest[string],
	tce *TestCaseExecutor,
	countdownEvents map[string]*CountdownEvent,
	reqRespSeq *sync.Map,
) (*protocol.CommandResponse[string], error) {
	for _, tcSync := range tce.Sync {
		if tcSync.WaitEvent != nil {
			err := countdownEvents[*tcSync.WaitEvent].Wait(ctx)
			if err != nil {
				return nil, err
			}
		}

		if tcSync.SignalEvent != nil {
			countdownEvents[*tcSync.SignalEvent].Signal()
		}
	}

	if tce.RaiseError {
		return nil, errors.New("error")
	}

	var response string
	responses, ok := tce.RequestResponsesMap[req.Payload]
	if ok && len(responses) > 0 {
		seqVal, ok := reqRespSeq.Load(req.Payload)
		if !ok {
			seqVal = 0
		}

		seqNo, ok := seqVal.(int)
		if !ok {
			seqNo = 0
		}

		reqRespSeq.Store(req.Payload, seqNo+1)
		response = responses[seqNo%len(responses)]
	}

	responseMetadata := make(map[string]string)
	for key, val := range tce.ResponseMetadata {
		if val != nil {
			responseMetadata[key] = *val
		} else {
			responseMetadata[key] = req.Metadata[key]
		}
	}

	if tce.TokenMetadataPrefix != nil {
		for key, val := range req.TopicTokens {
			responseMetadata[*tce.TokenMetadataPrefix+key] = val
		}
	}

	return &protocol.CommandResponse[string]{Message: protocol.Message[string]{
		Payload:  response,
		Metadata: responseMetadata,
	}}, nil
}
