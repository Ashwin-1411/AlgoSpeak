# Factorial computed recursively using algorithm/reveal
algorithm factorial(n)
    if n is less than or equal to 1
        reveal 1
    end
    reveal n * factorial(n - 1)
end

show factorial(5)
