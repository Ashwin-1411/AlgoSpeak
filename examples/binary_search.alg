# Binary search on a sorted array
create numbers as [1, 3, 5, 7, 9, 11]
create target as 7

create low as 0
create high as length of numbers minus 1
create result as -1

while low is less than or equal to high
    create mid as (low + high) / 2

    if numbers[mid] equals target
        set result to mid
        stop
    end

    if numbers[mid] is less than target
        set low to mid + 1
    otherwise
        set high to mid - 1
    end
end

show result
