import yaml
import json
import sys


def yaml_to_json(yaml_file, json_file):
	with open(yaml_file, 'r') as file:
		yaml_content = yaml.safe_load(file)

	with open(json_file, 'w') as file:
		json.dump(yaml_content, file, indent=2)
	print(f"Wrote to {json_file}")

def update_extension(yaml_path):
	split = yaml_path.split('.')
	if split[-1] != 'yaml':
		print("The input file is not a yaml file.")
		sys.exit(1)
	split[-1] = 'json'
	return '.'.join(split)

if __name__ == '__main__':
	if len(sys.argv) == 3:
		yaml_to_json(sys.argv[1], sys.argv[2])
	elif len(sys.argv) == 2:
		output_file = update_extension(sys.argv[1])
		yaml_to_json(sys.argv[1], output_file)
	else:
		print("Usage: python yaml_to_json.py <yaml_file> <json_file>")
		sys.exit(1)