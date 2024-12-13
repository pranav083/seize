import os
import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Define the file path
file_path = 'atomic_queue_memory_usage.csv'

# Check if the file exists
if not os.path.exists(file_path):
    raise FileNotFoundError(f"The file '{file_path}' does not exist. Please check the path and try again.")

# Load the data from CSV
data = pd.read_csv(file_path)

# Rename columns for easier handling if necessary
data.rename(columns={
    'Reclamation Scheme': 'scheme',
    'Operation': 'operation',
    'Memory Change (KB)': 'memory_change'
}, inplace=True)

# Remove ' KB' from memory-related columns and convert them to numeric
data['memory_change'] = data['memory_change'].str.replace(' KB', '').astype(float)

# Filter data for relevant schemes and operations
schemes = ['no scheme','ref_counting','seize','crossbeam','hazard_pointer']  # Adjust based on the schemes present in your dataset
operations = ['enqueue', 'dequeue']  # Adjust operations if necessary
data = data[data['scheme'].isin(schemes) & data['operation'].isin(operations)]

# Function to create a moving average for smoothing
def smooth_data(series, window=15):
    return series.rolling(window=window, min_periods=1).mean()

# Separate data by operation and smooth
insert_lines = []
delete_lines = []
for scheme in schemes:
    for operation in operations:
        subset = data[(data['scheme'] == scheme) & (data['operation'] == operation)].copy()
        subset.reset_index(drop=True, inplace=True)
        subset['smoothed_memory_change'] = smooth_data(subset['memory_change'], window=10)
        if operation == 'enqueue':
            insert_lines.append((subset, f"{scheme} ({operation})"))
        elif operation == 'dequeue':
            delete_lines.append((subset, f"{scheme} ({operation})"))

# Plot insert operations
plt.figure(figsize=(12, 8))
for subset, label in insert_lines:
    plt.plot(subset.index, subset['smoothed_memory_change'], label=label)
plt.xlabel('Index (Smoothed)')
plt.ylabel('Memory Change (KB)')
plt.title('Smoothed Change in Memory Over Time (Enqueue Operations)')
plt.legend()
plt.grid(True)
plt.show()

# Plot remove operations
plt.figure(figsize=(12, 8))
for subset, label in delete_lines:
    plt.plot(subset.index, subset['smoothed_memory_change'], label=label)
plt.xlabel('Index (Smoothed)')
plt.ylabel('Memory Change (KB)')
plt.title('Smoothed Change in Memory Over Time (Dequeue Operations)')
plt.legend()
plt.grid(True)
plt.show()