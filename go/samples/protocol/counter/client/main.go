// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

package main

import (
	"context"
	"log/slog"
	"os"
	"sync/atomic"
	"time"

	"github.com/Azure/iot-operations-sdks/go/mqtt"
	"github.com/Azure/iot-operations-sdks/go/protocol"
	"github.com/Azure/iot-operations-sdks/go/samples/protocol/counter/envoy/counter"
	"github.com/lmittmann/tint"
)

var telemetryCount int64

func main() {
	ctx := context.Background()
	slog.SetDefault(slog.New(tint.NewHandler(os.Stdout, &tint.Options{
		Level: slog.LevelDebug,
	})))
	app := must(protocol.NewApplication(protocol.WithLogger(slog.Default())))

	mqttClient := must(mqtt.NewSessionClientFromEnv(
		mqtt.WithLogger(slog.Default()),
	))
	counterServerID := os.Getenv("COUNTER_SERVER_ID")
	slog.Info("initialized MQTT client", "counter_server_id", counterServerID)

	if counterServerID == "" {
		panic("COUNTER_SERVER_ID must be set to the server's MQTT client ID")
	}

	client := must(counter.NewCounterClient(
		app,
		mqttClient,
		handleTelemetry,
	))
	defer client.Close()

	check(mqttClient.Start())
	check(client.Start(ctx))

	runCounterCommands(ctx, client, counterServerID)

	if telemetryCount == 15 {
		slog.Info("received expected number of telemetry messages", "count", telemetryCount)
	} else {
		slog.Error("unexpected number of telemetry messages received", "expected", 15, "actual", telemetryCount)
	}
}

func handleTelemetry(ctx context.Context, msg *protocol.TelemetryMessage[counter.TelemetryCollection]) error {
	atomic.AddInt64(&telemetryCount, 1)
	p := msg.Payload
	if p.CounterValue != nil {
		slog.Info("received telemetry", "counter_value", *p.CounterValue)
	}
	msg.Ack()
	return nil
}

func runCounterCommands(ctx context.Context, client *counter.CounterClient, serverID string) {
	resp := must(client.ReadCounter(ctx, serverID))
	slog.Info("read counter", "value", resp.Payload.CounterResponse)

	for range 15 {
		respIncr := must(client.Increment(ctx, serverID, counter.IncrementRequestPayload{
			IncrementValue: 1,
		}))
		slog.Info("increment", "value", respIncr.Payload.CounterResponse)
	}

	time.Sleep(10 * time.Second)
}

func check(e error) {
	if e != nil {
		panic(e)
	}
}

func must[T any](t T, e error) T {
	check(e)
	return t
}
