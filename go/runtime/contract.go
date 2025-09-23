package runtime

import (
	"gopkg.in/yaml.v3"
)

// IntentContract represents the internal structure of an intent contract
type IntentContract struct {
	Version  string         `yaml:"version"`
	Kind     string         `yaml:"kind"`
	Metadata ContractMetadata `yaml:"metadata"`
	Spec     IntentSpec     `yaml:"spec"`
}

type ContractMetadata struct {
	Name        string            `yaml:"name"`
	Description string            `yaml:"description,omitempty"`
	Labels      map[string]string `yaml:"labels,omitempty"`
}

type IntentSpec struct {
	IntentPatterns   []IntentPattern   `yaml:"intentPatterns"`
	Implementation   Implementation    `yaml:"implementation"`
	QualityOfService *QualityOfService `yaml:"qualityOfService,omitempty"`
}

type IntentPattern struct {
	Pattern     Pattern            `yaml:"pattern"`
	Constraints *PatternConstraints `yaml:"constraints,omitempty"`
}

type Pattern struct {
	Action     string                 `yaml:"action"`
	Parameters map[string]interface{} `yaml:",inline"`
}

type PatternConstraints struct {
	RequiredParameters   []string                     `yaml:"requiredParameters,omitempty"`
	ParameterConstraints map[string]ParameterConstraint `yaml:"parameterConstraints,omitempty"`
}

type ParameterConstraint struct {
	Type      string      `yaml:"type,omitempty"`
	EnumValues []string    `yaml:"enumValues,omitempty"`
	Min       *float64    `yaml:"min,omitempty"`
	Max       *float64    `yaml:"max,omitempty"`
}

type Implementation struct {
	Endpoint  Endpoint             `yaml:"endpoint"`
	Resources []ResourceRequirement `yaml:"resources,omitempty"`
}

type Endpoint struct {
	Type       string `yaml:"type"`
	Port       *int   `yaml:"port,omitempty"`
	Procedure  string `yaml:"procedure,omitempty"`
	URL        string `yaml:"url,omitempty"`
}

type ResourceRequirement struct {
	Type  string `yaml:"type"`
	Units string `yaml:"units"`
	Kind  string `yaml:"kind,omitempty"`
}

type QualityOfService struct {
	Latency      string `yaml:"latency,omitempty"`
	Availability string `yaml:"availability,omitempty"`
	Priority     string `yaml:"priority,omitempty"`
}

// ParseIntentContract parses YAML data into an IntentContract
func ParseIntentContract(data []byte) (*IntentContract, error) {
	var contract IntentContract
	if err := yaml.Unmarshal(data, &contract); err != nil {
		return nil, err
	}
	return &contract, nil
}

// ToProto converts the internal contract to protobuf format
func (c *IntentContract) ToProto() *nfa_intent_v1alpha.IntentContract {
	// This would be a complete conversion implementation
	// For brevity, returning a stub
	return &nfa_intent_v1alpha.IntentContract{
		Version: c.Version,
		Kind:    c.Kind,
		Metadata: &nfa_intent_v1alpha.Metadata{
			Name:        c.Metadata.Name,
			Description: c.Metadata.Description,
			Labels:      c.Metadata.Labels,
		},
		// Additional fields would be converted here
	}
}

// Validate checks if the contract is valid
func (c *IntentContract) Validate() error {
	if c.Version != "v1alpha" {
		return fmt.Errorf("unsupported version: %s", c.Version)
	}
	if c.Kind != "IntentContract" {
		return fmt.Errorf("invalid kind: %s", c.Kind)
	}
	if c.Metadata.Name == "" {
		return fmt.Errorf("metadata name is required")
	}
	if len(c.Spec.IntentPatterns) == 0 {
		return fmt.Errorf("at least one intent pattern is required")
	}
	return nil
}