# ML Engine Architecture

## Overview

Integrated ML engine for in-database training and inference, enabling SQL queries with ML capabilities.

## Supported Algorithms

- **Linear Regression**: Continuous value prediction
- **Logistic Regression**: Binary classification
- **K-Means**: Clustering
- **Decision Trees**: Classification and regression

## Training

- Integrated with SQL via `TRAIN MODEL` statement
- Supports in-database training on table data
- Feature engineering through SQL expressions
- Hyperparameter configuration

## Inference

- `PREDICT` SQL function for batch predictions
- Integrated into query execution
- Optimized inference pipeline

## Model Management

- Model catalog storage
- Version tracking
- Model metadata (schema, statistics)
- Persistence to disk

## Model Caching

- LRU cache for frequently used models
- In-memory inference acceleration
- Cache statistics and monitoring

## Security

- Model encryption
- Access control
- Audit logging for model usage
