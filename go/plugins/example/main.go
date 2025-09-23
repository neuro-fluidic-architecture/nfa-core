package main

import (
	"context"
	"fmt"
	"log"
	"os"

	"github.com/neuro-fluidic-architecture/nfa-core/go/runtime"
	nfa_intent_v1alpha "github.com/neuro-fluidic-architecture/nfa-core/go/protos/intent/v1alpha"
	"google.golang.org/grpc"
)

// ExampleService 是一个示例意图服务实现
type ExampleService struct {
	nfa_intent_v1alpha.UnimplementedExampleServiceServer
}

// ProcessExample 处理示例意图
func (s *ExampleService) ProcessExample(ctx context.Context, req *nfa_intent_v1alpha.ExampleRequest) (*nfa_intent_v1alpha.ExampleResponse, error) {
	log.Printf("Processing example request: %s", req.Input)
	
	// 简单的处理逻辑
	result := fmt.Sprintf("Processed: %s (length: %d)", req.Input, len(req.Input))
	
	return &nfa_intent_v1alpha.ExampleResponse{
		Result:    result,
		Processed: true,
		Metadata:  map[string]string{"input_length": fmt.Sprintf("%d", len(req.Input))},
	}, nil
}

func main() {
	// 连接到Broker
	brokerAddr := getEnv("NFA_BROKER_ADDRESS", "localhost:50051")
	runtime := runtime.NewIntentRuntime(brokerAddr)
	
	if err := runtime.Connect(); err != nil {
		log.Fatalf("Failed to connect to broker: %v", err)
	}
	defer runtime.Close()
	
	// 注册意图服务
	contractPath := getEnv("NFA_SERVICE_CONTRACT_PATH", "example.intent.yaml")
	serviceID, err := runtime.RegisterFromFile(contractPath)
	if err != nil {
		log.Fatalf("Failed to register service: %v", err)
	}
	
	log.Printf("Service registered with ID: %s", serviceID)
	
	// 创建gRPC服务器
	server := runtime.NewIntentServer(50052)
	exampleService := &ExampleService{}
	
	// 注册示例服务
	nfa_intent_v1alpha.RegisterExampleServiceServer(server.Server, exampleService)
	
	// 启动健康报告
	go runtime.StartHealthReporting()
	
	// 启动服务器
	log.Printf("Starting example service on port %d", server.GetPort())
	if err := server.Start(); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}

func getEnv(key, defaultValue string) string {
	if value, exists := os.LookupEnv(key); exists {
		return value
	}
	return defaultValue
}