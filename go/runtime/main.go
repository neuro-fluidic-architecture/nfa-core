package main

import (
	"context"
	"flag"
	"fmt"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/neuro-fluidic-architecture/nfa-core/go/runtime"
)

func main() {
	// 解析命令行参数
	brokerAddr := flag.String("broker", "localhost:50051", "Broker address")
	contractPath := flag.String("contract", "", "Path to intent contract YAML file")
	servicePort := flag.Int("port", 0, "Service port (0 for auto)")
	flag.Parse()

	// 检查必需参数
	if *contractPath == "" {
		log.Fatal("Contract path is required")
	}

	// 创建运行时实例
	rt := runtime.NewIntentRuntime(*brokerAddr)
	
	// 连接到Broker
	if err := rt.Connect(); err != nil {
		log.Fatalf("Failed to connect to broker: %v", err)
	}
	defer rt.Close()

	// 注册意图服务
	serviceID, err := rt.RegisterFromFile(*contractPath)
	if err != nil {
		log.Fatalf("Failed to register service: %v", err)
	}

	log.Printf("Service registered with ID: %s", serviceID)

	// 启动健康报告
	go rt.StartHealthReporting()

	// 创建gRPC服务器
	server := runtime.NewIntentServer(*servicePort)
	
	// 这里可以注册服务实现
	// 例如: translator.RegisterTranslatorServer(server, &translator.TranslatorService{})
	
	// 启动服务器
	go func() {
		log.Printf("Starting server on port %d", server.GetPort())
		if err := server.Start(); err != nil {
			log.Fatalf("Failed to start server: %v", err)
		}
	}()

	// 等待终止信号
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
	
	<-sigChan
	log.Println("Shutting down...")
	
	// 优雅关闭
	server.Stop()
	
	log.Println("Service stopped")
}