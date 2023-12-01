import struct
import os
import re
import numpy as np
import sys

pattern = re.compile(r".*\.o")
current_folder = os.getcwd()

file_paths = [file for file in os.listdir(current_folder) if pattern.match(file)]

length = len(file_paths)

print(file_paths)
print(length)

merged_file = open("output/apps.bin", "wb")
merged_file.truncate(32*1024*1024)

offset_content = 8 * (len(file_paths) * 2 + 1)
print(offset_content)

merged_file.seek(0, 0)
merged_file.write(struct.pack('<Q', length))
offset_header = 8
i = 0
for file_name in file_paths:
    fsize = os.path.getsize(file_name)
    merged_file.seek(offset_header, 0)
    merged_file.write(struct.pack('<Q', offset_content))
    merged_file.write(struct.pack('<Q', fsize))
    merged_file.seek(offset_content, 0)
    with open(file_name, "rb") as file:
        # 读取每个文件的内容，并将其写入合并文件中
        merged_file.write(file.read())
    offset_header += 16
    offset_content += fsize

merged_file.close()