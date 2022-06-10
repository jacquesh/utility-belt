#include <stdio.h>
#include <stdlib.h>

int main(int argc, const char** argv)
{
	if(argc <= 1)
	{
		fprintf(stderr, "No input file provided\n");
		return 1;
	}

	if((strcmp(argv[1], "-h") == 0) || (strcmp(argv[1], "--help") == 0))
	{
		fprintf(stderr, "Usage: %s FILE\n", argv[0]);
		fprintf(stderr, "Check to see if the tabs in the given source code file make sense\n");
		fprintf(stderr, "Tabs do not make sense if they appear after the first non-tab character on a line\n");
		fprintf(stderr, "\n");
		fprintf(stderr, "e.g Good: 'tab tab a space b'\n");
		fprintf(stderr, "    Bad: 'tab tab a tab b'\n");
		fprintf(stderr, "    Bad: 'space tab a space b'\n");
		return 0;
	}

	const char* input_file_path = argv[1];
	FILE* input_file = fopen(input_file_path, "rb");
	if(input_file == NULL)
	{
		printf("Failed to open input file\n");
		return 1;
	}
	fseek(input_file, 0, SEEK_END);
	const size_t file_length = ftell(input_file);
	fseek(input_file, 0, SEEK_SET);

	char* data = (char*)malloc(file_length);
	size_t bytes_read = 0;
	while(bytes_read < file_length)
	{
		size_t new_bytes = fread(data + bytes_read, 1, file_length - bytes_read, input_file);
		if(new_bytes == 0)
		{
			printf("Failed to read input file. Terminating...\n");
			return 1;
		}
		bytes_read += new_bytes;
	}

	int line_has_nontab = 0;
	int line_error_printed = 0;
	size_t line_num = 1;
	size_t line_start_index = 0;
	int error_count = 0;
	for(size_t index=0; index<file_length; index++)
	{
		const char val = data[index];

		if(!line_has_nontab && (val != '\t'))
		{
			line_has_nontab = 1;
		}

		if(line_has_nontab && !line_error_printed && (val == '\t'))
		{
			const size_t column_num = index - line_start_index + 1;
			printf("line %ju: disallowed tab at byte %ju\n", line_num, column_num);
			line_error_printed = 1;
			error_count++;
		}

		if(val == '\n')
		{
			line_has_nontab = 0;
			line_error_printed = 0;
			line_start_index = index+1;
			line_num++;
		}
	}

    printf("total: %d error lines\n", error_count);
	return 0;
}
