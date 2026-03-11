# Sort and reverse demonstration in AlgoSpeak
# Shows built-in sort and reverse standard library functions

create numbers as [5, 3, 8, 1, 9, 2]
create n as length of numbers

# Sort the array
sort numbers

# Display sorted array: should show 1 2 3 5 8 9
create i as 0
while i is less than n
    show numbers[i]
    set i to i + 1
end
