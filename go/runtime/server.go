package runtime

import (
	"context"
	"fmt"
	"log"
	"net"

	"google.golang.org/grpc"
	"google.golang.org/grpc/health"
	"google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/reflection"
)

// IntentServer hosts the gRPC server for intent service implementations
type IntentServer struct {
	server   *grpc.Server
	services map[string]interface{} // service name -> implementation
	port     int
}

// NewIntentServer creates a new intent server
func NewIntentServer(port int) *IntentServer {
	return &IntentServer{
		server:   grpc.NewServer(),
		services: make(map[string]interface{}),
		port:     port,
	}
}

// RegisterService registers a service implementation
func (s *IntentServer) RegisterService(desc *grpc.ServiceDesc, impl interface{}) {
	s.server.RegisterService(desc, impl)
	s.services[desc.ServiceName] = impl
	log.Printf("Registered service: %s", desc.ServiceName)
}

// Start starts the gRPC server
func (s *IntentServer) Start() error {
	// Register health service
	healthServer := health.NewServer()
	grpc_health_v1.RegisterHealthServer(s.server, healthServer)
	
	// Register reflection service
	reflection.Register(s.server)

	// Start server
	lis, err := net.Listen("tcp", fmt.Sprintf(":%d", s.port))
	if err != nil {
		return fmt.Errorf("failed to listen: %v", err)
	}

	log.Printf("Server listening on port %d", s.port)
	
	// Update health status for all services
	for serviceName := range s.services {
		healthServer.SetServingStatus(serviceName, grpc_health_v1.HealthCheckResponse_SERVING)
	}

	return s.server.Serve(lis)
}

// Stop gracefully stops the server
func (s *IntentServer) Stop() {
	log.Println("Shutting down server...")
	s.server.GracefulStop()
	log.Println("Server stopped")
}

// GetPort returns the server port
func (s *IntentServer) GetPort() int {
	return s.port
}