"""
Neural Network Model for Shogi AI

Defines a simple ResNet-style model with Policy and Value heads.
"""

import torch
import torch.nn as nn
import torch.nn.functional as F


class ResBlock(nn.Module):
    """Residual block"""
    def __init__(self, channels):
        super().__init__()
        self.conv1 = nn.Linear(channels, channels)
        self.conv2 = nn.Linear(channels, channels)
        self.bn1 = nn.BatchNorm1d(channels)
        self.bn2 = nn.BatchNorm1d(channels)
    
    def forward(self, x):
        residual = x
        out = F.relu(self.bn1(self.conv1(x)))
        out = self.bn2(self.conv2(out))
        out += residual
        out = F.relu(out)
        return out


class ShogiNet(nn.Module):
    """
    Simple neural network for Shogi evaluation.
    
    Architecture:
    - Input: Board features (2210 dims)
    - Body: Several ResBlocks
    - Policy Head: Move probabilities
    - Value Head: Win probability
    """
    def __init__(self, input_size=2647, hidden_size=256, num_blocks=4, num_actions=7290):
        super().__init__()
        
        # Input layer
        self.input_layer = nn.Linear(input_size, hidden_size)
        self.input_bn = nn.BatchNorm1d(hidden_size)
        
        # Body (residual blocks)
        self.res_blocks = nn.ModuleList([
            ResBlock(hidden_size) for _ in range(num_blocks)
        ])
        
        # Policy head
        self.policy_head = nn.Sequential(
            nn.Linear(hidden_size, hidden_size // 2),
            nn.ReLU(),
            nn.Linear(hidden_size // 2, num_actions)
        )
        
        # Value head
        self.value_head = nn.Sequential(
            nn.Linear(hidden_size, hidden_size // 4),
            nn.ReLU(),
            nn.Linear(hidden_size // 4, 1),
            nn.Tanh()  # Output in [-1, 1]
        )
    
    def forward(self, x):
        # Input
        x = F.relu(self.input_bn(self.input_layer(x)))
        
        # Body
        for block in self.res_blocks:
            x = block(x)
        
        # Heads
        policy = self.policy_head(x)
        value = self.value_head(x)
        
        return policy, value


if __name__ == '__main__':
    # Test model
    model = ShogiNet()
    dummy_input = torch.randn(4, 2647)  # Batch of 4
    policy, value = model(dummy_input)
    
    print(f"Policy shape: {policy.shape}")  # [4, 7290]
    print(f"Value shape: {value.shape}")    # [4, 1]
    print("Model test passed!")
