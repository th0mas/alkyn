#! /usr/bin/python3

def starts_with_1(line):
  return line[0] == "1"

log_file = open("log", "r")
lines = log_file.read().split('\n')[0::2]

sorted_lines = filter(starts_with_1, lines)
print("\n".join(sorted_lines))

