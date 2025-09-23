package main

import (
	"context"
	"fmt"
	"log"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/health"
	"google.golang.org/grpc/health/grpc_health_v1"

	"github.com/neuro-fluidic-architecture/nfa-core/go/runtime"
	nfa_intent_v1alpha "github.com/neuro-fluidic-architecture/nfa-core/go/protos/intent/v1alpha"
)

// TranslatorService implements the translation service
type TranslatorService struct {
	nfa_intent_v1alpha.UnimplementedTranslatorServer
}

// TranslateText implements the translation RPC
func (s *TranslatorService) TranslateText(ctx context.Context, req *nfa_intent_v1alpha.TranslateRequest) (*nfa_intent_v1alpha.TranslateResponse, error) {
	log.Printf("Translating text: %s from %s to %s", req.Text, req.SourceLanguage, req.TargetLanguage)
	
	// Simple translation logic - in real implementation, this would use a translation library/API
	translations := map[string]map[string]string{
		"hello": {
			"zh": "你好",
			"fr": "bonjour",
			"de": "hallo",
			"es": "hola",
		},
		"world": {
			"zh": "世界",
			"fr": "monde",
			"de": "welt",
			"es": "mundo",
		},
	}
	
	// Simple word-by-word translation
	var translatedText string
	if translation, exists := translations[req.Text]; exists {
		if translated, exists := translation[req.TargetLanguage]; exists {
			translatedText = translated
		} else {
			translatedText = req.Text // Fallback to original text
		}
	} else {
		translatedText = req.Text // Fallback to original text
	}
	
	return &nfa_intent_v1alpha.TranslateResponse{
		TranslatedText: translatedText,
		SourceLanguage: req.SourceLanguage,
		TargetLanguage: req.TargetLanguage,
	}, nil
}

func main() {
	// Create and connect runtime
	runtime := runtime.NewIntentRuntime("localhost:50051")
	if err := runtime.Connect(); err != nil {
		log.Fatalf("Failed to connect to broker: %v", err)
	}
	defer runtime.Close()
	
	// Register the intent service
	serviceID, err := runtime.RegisterFromFile("translator.intent.yaml")
	if err != nil {
		log.Fatalf("Failed to register service: %v", err)
	}
	
	log.Printf("Service registered with ID: %s", serviceID)
	
	// Start health reporting
	go runtime.StartHealthReporting()
	
	// Create and start gRPC server
	server := runtime.NewIntentServer(50052)
	translatorService := &TranslatorService{}
	
	// Register the translator service
	nfa_intent_v1alpha.RegisterTranslatorServer(server.Server, translatorService)
	
	// Start the server
	if err := server.Start(); err != nil {
		log.Fatalf("Failed to start server: %v", err)
	}
}