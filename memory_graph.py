import pandas as pd
import matplotlib.pyplot as plt
import numpy as np

# Load the data from CSV
# Replace 'atomic_queue_memory_usage.csv' with the actual path to your CSV file
data = pd.read_csv('lockfree_list_memory_usage.csv')

# Rename columns for easier handling if necessary
data.rename(columns={
    'Reclamation Scheme': 'scheme',
    'Operation': 'operation',
    'Memory Change (KB)': 'memory_change'
}, inplace=True)

# Remove 'KB' from the 'memory_change' column and convert it to numeric
data['memory_change'] = data['memory_change'].str.replace(' KB', '').astype(float)

# Filter data for relevant schemes and operations (adjust based on your needs)
schemes = ['ref_counting', 'seize', 'crossbeam', 'hazard_pointer']  # Add or modify schemes as needed
operations = ['insert', 'remove']  # Adjust operations if necessary
data = data[data['scheme'].isin(schemes) & data['operation'].isin(operations)]

# Aggregate data to reduce density (e.g., average every 10 rows)
def aggregate_data(subset, interval=10):
    numeric_columns = subset.select_dtypes(include=[np.number])
    aggregated = numeric_columns.groupby(numeric_columns.index // interval).mean()
    return aggregated

# Separate data by operation and aggregate
insert_lines = []
delete_lines = []
for scheme in schemes:
    for operation in operations:
        subset = data[(data['scheme'] == scheme) & (data['operation'] == operation)].copy()
        subset.reset_index(drop=True, inplace=True)
        aggregated = aggregate_data(subset)
        if operation == 'insert':
            insert_lines.append((aggregated, f"{scheme} ({operation})"))
        else:
            delete_lines.append((aggregated, f"{scheme} ({operation})"))

# Plot insert operations
plt.figure(figsize=(12, 8))
for subset, label in insert_lines:
    plt.plot(subset.index, subset['memory_change'], label=label)
plt.xlabel('Index (aggregated)')
plt.ylabel('Memory Change (KB)')
plt.title('Change in Memory Over Time (Insert Operations)')
plt.legend()
plt.grid(True)
plt.show()

# Plot delete operations
plt.figure(figsize=(12, 8))
for subset, label in delete_lines:
    plt.plot(subset.index, subset['memory_change'], label=label)
plt.xlabel('Index (aggregated)')
plt.ylabel('Memory Change (KB)')
plt.title('Change in Memory Over Time (remove Operations)')
plt.legend()
plt.grid(True)
plt.show()
