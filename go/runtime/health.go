package runtime

import (
	"context"
	"fmt"
	"time"

	"google.golang.org/grpc/health/grpc_health_v1"
)

// HealthChecker implements gRPC health check service
type HealthChecker struct {
	runtime *IntentRuntime
}

// NewHealthChecker creates a new health checker
func NewHealthChecker(runtime *IntentRuntime) *HealthChecker {
	return &HealthChecker{
		runtime: runtime,
	}
}

// Check implements the health check RPC
func (h *HealthChecker) Check(ctx context.Context, req *grpc_health_v1.HealthCheckRequest) (*grpc_health_v1.HealthCheckResponse, error) {
	// Check if runtime is connected to broker
	if h.runtime.conn == nil {
		return &grpc_health_v1.HealthCheckResponse{
			Status: grpc_health_v1.HealthCheckResponse_NOT_SERVING,
		}, nil
	}

	// Check broker connection state
	state := h.runtime.conn.GetState()
	if state.String() != "READY" {
		return &grpc_health_v1.HealthCheckResponse{
			Status: grpc_health_v1.HealthCheckResponse_NOT_SERVING,
		}, nil
	}

	return &grpc_health_v1.HealthCheckResponse{
		Status: grpc_health_v1.HealthCheckResponse_SERVING,
	}, nil
}

// Watch implements the health watch RPC
func (h *HealthChecker) Watch(req *grpc_health_v1.HealthCheckRequest, stream grpc_health_v1.Health_WatchServer) error {
	// Simple implementation - just send current status periodically
	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-stream.Context().Done():
			return nil
		case <-ticker.C:
			status, err := h.Check(stream.Context(), req)
			if err != nil {
				return err
			}
			if err := stream.Send(status); err != nil {
				return err
			}
		}
	}
}

// StartHealthReporting starts periodic health reporting to the broker
func (r *IntentRuntime) StartHealthReporting() {
	if r.serviceID == "" {
		return // Not registered yet
	}

	ticker := time.NewTicker(10 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			if err := r.sendHeartbeat(); err != nil {
				fmt.Printf("Heartbeat failed: %v\n", err)
			}
		}
	}
}

func (r *IntentRuntime) sendHeartbeat() error {
	if r.client == nil {
		return fmt.Errorf("not connected to broker")
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	_, err := r.client.Heartbeat(ctx, &protos.HeartbeatRequest{
		ServiceId: r.serviceID,
	})

	return err
}