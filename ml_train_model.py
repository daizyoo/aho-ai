#!/usr/bin/env python3
"""
Simple Neural Network for Move Prediction
Trains a model to predict next moves from game positions
"""

import pickle
import numpy as np
from pathlib import Path

try:
    import tensorflow as tf
    from tensorflow import keras
    HAS_TF = True
except ImportError:
    HAS_TF = False
    print("TensorFlow not installed. Install with: pip install tensorflow")

def build_move_prediction_model(input_dim: int = 7, hidden_units: int = 128) -> 'keras.Model':
    """Build a simple feedforward network for move prediction"""
    model = keras.Sequential([
        keras.layers.Dense(hidden_units, activation='relu', input_shape=(input_dim,)),
        keras.layers.Dropout(0.2),
        keras.layers.Dense(hidden_units, activation='relu'),
        keras.layers.Dropout(0.2),
        keras.layers.Dense(hidden_units // 2, activation='relu'),
        keras.layers.Dense(input_dim, activation='sigmoid')  # Predict next move features
    ])
    
    model.compile(
        optimizer='adam',
        loss='mse',
        metrics=['mae']
    )
    
    return model

def prepare_sequences_for_training(sequences, window_size: int = 5):
    """Convert sequences to (X, y) pairs for supervised learning"""
    X, y = [], []
    
    for sequence in sequences:
        if len(sequence) < window_size + 1:
            continue
        
        for i in range(len(sequence) - window_size):
            # Use last N moves to predict next move
            window = sequence[i:i+window_size]
            next_move = sequence[i+window_size]
            
            # Flatten window
            X.append(np.concatenate(window))
            y.append(next_move)
    
    return np.array(X), np.array(y)

def train_model(data_file: str = 'training_data.pkl', epochs: int = 50):
    """Train the move prediction model"""
    if not HAS_TF:
        print("Cannot train without TensorFlow")
        return
    
    # Load data
    with open(data_file, 'rb') as f:
        data = pickle.load(f)
    
    print(f"Loaded {data['num_games']} games")
    
    # Prepare training data
    window_size = 5
    X, y = prepare_sequences_for_training(data['sequences'], window_size)
    
    print(f"Training samples: {len(X)}")
    print(f"Input shape: {X.shape}")
    print(f"Output shape: {y.shape}")
    
    # Build model
    input_dim = window_size * 7  # 7 features per move
    model = build_move_prediction_model(input_dim)
    
    print("\nModel Summary:")
    model.summary()
    
    # Train
    history = model.fit(
        X, y,
        epochs=epochs,
        batch_size=32,
        validation_split=0.2,
        verbose=1
    )
    
    # Save model
    model.save('move_prediction_model.h5')
    print("\nModel saved to: move_prediction_model.h5")
    
    return model, history

def main():
    import sys
    
    if not HAS_TF:
        print("\nTo use this script, install TensorFlow:")
        print("  pip install tensorflow")
        sys.exit(1)
    
    data_file = 'training_data.pkl'
    
    if not Path(data_file).exists():
        print(f"Error: {data_file} not found")
        print("Run ml_prepare_data.py first to create training data")
        sys.exit(1)
    
    epochs = int(sys.argv[1]) if len(sys.argv) > 1 else 50
    
    print(f"Training for {epochs} epochs...\n")
    train_model(data_file, epochs)

if __name__ == "__main__":
    main()
