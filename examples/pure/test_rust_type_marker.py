"""Test RustType marker functionality"""
import pure

# Test 1: process_container
container = pure.DataContainer(42)
result = pure.process_container(container)
assert isinstance(result, pure.DataContainer)
assert result.value == 84

# Test 2: sum_list
numbers = [1, 2, 3, 4, 5]
total = pure.sum_list(numbers)
assert total == 15

# Test 3: create_containers
containers = pure.create_containers(3)
assert len(containers) == 3
assert all(isinstance(c, pure.DataContainer) for c in containers)

# Test 4: Calculator multiply
calc1 = pure.Calculator()
calc1.add(5.0)
calc2 = pure.Calculator()
calc2.add(3.0)
result_calc = calc1.multiply(calc2)
assert isinstance(result_calc, pure.Calculator)

print("All tests passed!")
