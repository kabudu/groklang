import time

def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)

def main():
    start = time.time()
    result = fib(30)
    end = time.time()
    print(f"Result: {result}")
    print(f"Time: {end - start:.4f}s")

if __name__ == "__main__":
    main()
