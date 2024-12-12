import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Load the data from CSV
# Replace 'data.csv' with the actual path to your CSV file
data = pd.read_csv('atomic_queue_memory_usage.csv')

# Rename columns for easier handling if necessary
data.rename(columns={
    'Reclamation Scheme': 'scheme',
    'Operation': 'operation',
    'Memory Free Change (KB)': 'free_memory',
    'Memory Before (KB)': 'memory_before',
    'Memory After (KB)': 'memory_after'
}, inplace=True)

# Remove 'KB' from relevant columns and convert to numeric
data['free_memory'] = data['free_memory'].str.replace(' KB', '').astype(float)
data['memory_before'] = data['memory_before'].str.replace(' KB', '').astype(float)
data['memory_after'] = data['memory_after'].str.replace(' KB', '').astype(float)

# Filter data for relevant schemes and operations
schemes = ['ref_counting', 'seize', 'crossbeam', 'hazard_pointer']
operations = ['enqueue', 'dequeue']
data = data[data['scheme'].isin(schemes) & data['operation'].isin(operations)]

# Aggregate data to reduce density (e.g., average every 10 rows)
def aggregate_data(subset, interval=10):
    numeric_columns = subset.select_dtypes(include=[np.number])
    aggregated = numeric_columns.groupby(numeric_columns.index // interval).mean()
    return aggregated

# Separate data by operation and aggregate
enqueue_lines = []
dequeue_lines = []
for scheme in schemes:
    for operation in operations:
        subset = data[(data['scheme'] == scheme) & (data['operation'] == operation)].copy()
        subset.reset_index(drop=True, inplace=True)
        aggregated = aggregate_data(subset)
        if operation == 'enqueue':
            enqueue_lines.append((aggregated, f"{scheme} ({operation})"))
        else:
            dequeue_lines.append((aggregated, f"{scheme} ({operation})"))

# Plot enqueue operations
plt.figure(figsize=(12, 8))
for subset, label in enqueue_lines:
    plt.plot(subset.index, subset['free_memory'], label=label)
plt.xlabel('Index (aggregated)')
plt.ylabel('Memory Free Change (KB)')
plt.title('Change in Free Memory Over Time (Enqueue Operations)')
plt.legend()
plt.grid(True)
plt.show()

# Plot dequeue operations
plt.figure(figsize=(12, 8))
for subset, label in dequeue_lines:
    plt.plot(subset.index, subset['free_memory'], label=label)
plt.xlabel('Index (aggregated)')
plt.ylabel('Memory Free Change (KB)')
plt.title('Change in Free Memory Over Time (Dequeue Operations)')
plt.legend()
plt.grid(True)
plt.show()
