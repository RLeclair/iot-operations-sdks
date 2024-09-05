package main

import (
	"bufio"
	"context"
	"flag"
	"fmt"
	"log/slog"
	"os"
	"time"

	"github.com/Azure/iot-operations-sdks/go/mqtt"
	"github.com/Azure/iot-operations-sdks/go/protocol"
	"github.com/Azure/iot-operations-sdks/go/protocol/iso"
	"github.com/Azure/iot-operations-sdks/go/samples/greeter/envoy"
	"github.com/lmittmann/tint"
)

func main() {
	ctx := context.Background()
	slog.SetDefault(slog.New(tint.NewHandler(os.Stdout, nil)))

	// EnableManualAcknowledgment must be set.
	clientID := fmt.Sprintf("sampleClient-%d", time.Now().UnixMilli())
	connStr := fmt.Sprintf("ClientID=%s;HostName=%s;TcpPort=%s",
		clientID,
		"localhost",
		"1883",
	)
	mqttClient := must(mqtt.NewSessionClientFromConnectionString(connStr))
	check(mqttClient.Connect(ctx))

	client := must(envoy.NewGreeterClient(mqttClient))
	done := must(client.Listen(ctx))
	defer done()

	n := flag.String("n", "User", "the name to greet (default: User)")
	d := flag.String("d", "", "how long to delay (in Go duration format)")
	i := flag.Bool("i", false, "whether to run interactively")
	flag.Parse()

	if *i {
		fmt.Println("Enter an empty name to quit.")
		fmt.Println("Optionally enter delays in Go duration format (e.g. 10s).")
		scanner := bufio.NewScanner(os.Stdin)
		for {
			fmt.Print("\nName: ")
			scanner.Scan()
			name := scanner.Text()
			if name == "" {
				return
			}

			fmt.Print("Delay: ")
			scanner.Scan()
			delay := scanner.Text()

			// Call and wait a moment for any immediate response.
			go call(ctx, client, name, delay)
			time.Sleep(time.Second)
		}
	} else {
		call(ctx, client, *n, *d)
	}
}

func call(
	ctx context.Context,
	client *envoy.GreeterClient,
	name, delay string,
) {
	req := envoy.HelloRequest{Name: name}

	var res *protocol.CommandResponse[envoy.HelloResponse]
	if delay == "" {
		res = must(client.SayHello(ctx, req))
	} else {
		duration := must(time.ParseDuration(delay))
		delayReq := envoy.HelloWithDelayRequest{
			HelloRequest: req,
			Delay:        iso.Duration(duration),
		}
		res = must(client.SayHelloWithDelay(ctx, delayReq,
			protocol.WithMessageExpiry(10+uint32(duration.Seconds()))))
	}
	slog.Info(res.Payload.Message, slog.String("id", res.CorrelationData))
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
