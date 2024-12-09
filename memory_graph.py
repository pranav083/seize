import pandas as pd
import matplotlib.pyplot as plt

# Load memory usage data
data = pd.read_csv("memory_usage.csv", names=["Run ID", "Time (s)", "Available Memory (MB)"])

# Downsample the data for visualization (optional)
# For instance, keep every 10th sample
data = data.iloc[::10, :]

# Plot each run separately
plt.figure(figsize=(10, 6))
for run_id in data["Run ID"].unique():
    run_data = data[data["Run ID"] == run_id]
    plt.plot(run_data["Time (s)"], run_data["Available Memory (MB)"], label=f"Run {run_id}")

avg_memory = data.groupby("Time (s)")["Available Memory (MB)"].mean()
plt.plot(avg_memory.index, avg_memory.values, label="Average", linewidth=2, color="black")


plt.legend(title="Run ID")
plt.title("Free Memory vs Time (More Frequent Sampling)")
plt.xlabel("Time (s)")
plt.ylabel("Available Memory (MB)")
plt.show()
