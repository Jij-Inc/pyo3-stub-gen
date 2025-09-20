# Failing example for `pure.np_type_must_match`

import pure
import numpy as np

python_int_list: list[int] = [1, 2, 131_072]
np_int_array = np.array([1, 2, 131_072], dtype=np.int32)
np_f32_array = np.array([1.0, 2.0, 131_072.0], dtype=np.float32)
tuple_int_list: tuple[int, int, int] = (1, 2, 131_072)
tuple_float_list: tuple[float, float, float] = (1.0, 2.0, 131_072.0)

pure.np_type_must_match(python_int_list)
pure.np_type_must_match(np_int_array)
pure.np_type_must_match(np_f32_array)
pure.np_type_must_match(tuple_int_list)
pure.np_type_must_match(tuple_float_list)
