/* This is an auto-generated file.  Do not modify. */
package dtmi_com_example_Counter__1

import (
	"context"

	"github.com/Azure/iot-operations-sdks/go/protocol"
	"github.com/Azure/iot-operations-sdks/go/protocol/mqtt"
)

type IncrementCommandInvoker struct {
	*protocol.CommandInvoker[any, IncrementCommandResponse]
}

func NewIncrementCommandInvoker(
	client mqtt.Client,
	requestTopic string,
	opt ...protocol.CommandInvokerOption,
) (*IncrementCommandInvoker, error) {
	var err error
	invoker := &IncrementCommandInvoker{}

	var opts protocol.CommandInvokerOptions
	opts.Apply(
		opt,
		protocol.WithTopicTokenNamespace("ex:"),
		protocol.WithTopicTokens{
			"commandName":     "increment",
			"invokerClientId": client.ClientID(),
		},
	)

	invoker.CommandInvoker, err = protocol.NewCommandInvoker(
		client,
		protocol.Empty{},
		protocol.JSON[IncrementCommandResponse]{},
		requestTopic,
		&opts,
	)

	return invoker, err
}

func (invoker IncrementCommandInvoker) Increment(
	ctx context.Context,
	executorId string,
	opt ...protocol.InvokeOption,
) (*protocol.CommandResponse[IncrementCommandResponse], error) {
	var opts protocol.InvokeOptions
	opts.Apply(
		opt,
		protocol.WithTopicTokens{
			"executorId": executorId,
		},
	)

	response, err := invoker.Invoke(
		ctx,
		nil,
		&opts,
	)

	return response, err
}
