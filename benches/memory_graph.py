import pandas as pd
import matplotlib.pyplot as plt
from datetime import datetime

# Read the CSV file
df = pd.read_csv('lockfree_hash_map_memory_usage.csv', parse_dates=['Timestamp'])

# Convert memory values from KB to MB for better readability
df['Memory Before (MB)'] = df['Memory Before (KB)'] / 1024
df['Memory After (MB)'] = df['Memory After (KB)'] / 1024

# Calculate the time elapsed in seconds from the first timestamp
df['Time Elapsed'] = (df['Timestamp'] - df['Timestamp'].iloc[0]).dt.total_seconds()

# Create the plot
plt.figure(figsize=(12, 6))
plt.plot(df['Time Elapsed'], df['Memory Before (MB)'], label='Memory Before', marker='o')
plt.plot(df['Time Elapsed'], df['Memory After (MB)'], label='Memory After', marker='s')

# Customize the plot
plt.title('Memory Usage Over Time for Lock-free List with Reference Counting')
plt.xlabel('Time Elapsed (seconds)')
plt.ylabel('Memory Usage (MB)')
plt.legend()
plt.grid(True)

# Add annotations for insert operations
for i, row in df.iterrows():
    plt.annotate('Insert', (row['Time Elapsed'], row['Memory After (MB)']),
                 xytext=(5, 5), textcoords='offset points', fontsize=8, alpha=0.7)

# Improve layout
plt.tight_layout()

# Save the plot as an image file
plt.savefig('memory_usage_graph.png', dpi=300)

# Display the plot (optional, comment out if running in a non-interactive environment)
plt.show()
