# A simple sum program: adds all numbers in an array
create numbers as [1, 3, 5, 7, 9]

create total as 0

for each number in numbers
    add number to total
end

show total
