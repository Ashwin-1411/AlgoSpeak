# Bubble sort — sorts an array in place
create numbers as [5, 3, 8, 1, 9, 2]
create n as length of numbers

create i as 0
while i is less than n
    create j as 0
    while j is less than n minus 1 minus i
        if numbers[j] is greater than numbers[j + 1]
            create temp as numbers[j]
            set numbers[j] to numbers[j + 1]
            set numbers[j + 1] to temp
        end
        set j to j + 1
    end
    set i to i + 1
end

# Show the sorted array
create k as 0
while k is less than n
    show numbers[k]
    set k to k + 1
end
