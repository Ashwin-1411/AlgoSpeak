# Queue operations in AlgoSpeak
# Demonstrates the built-in queue data structure

create queue q

enqueue 100 into q
enqueue 200 into q
enqueue 300 into q

# Dequeue and display: should show 100, 200, 300 (FIFO order)
show dequeue from q
show dequeue from q
show dequeue from q
