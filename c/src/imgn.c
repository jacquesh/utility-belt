// NOTE: This tool (imgn, pronounced "imagine") was written because occasionally I want to do batch
//       resizing or converting of images and the commonly-used tool for that is ImageMagick but
//       not only does that not appear to have decent batch processing support, it also has several
//       vulnerabilities: https://www.enisa.europa.eu/publications/info-notes/what2019s-behind-imagemagick-vulnerability
#include <stdlib.h>
#include <stdio.h>
#include <stdint.h>

// TODO: I'm not really a fan of kgflags. It doesn't let you specify short versions of flags
//       (e.g -o instead of --output), it doesn't have builtin support for -h/--help and it
//       has weird formatting on its builtin help page (e.g there appears to be a random amount
//       of spacing after a flag and before its type).
#define KGFLAGS_IMPLEMENTATION
#include "kgflags.h"

#define STB_IMAGE_IMPLEMENTATION
#include "stb_image.h"

#define STB_IMAGE_RESIZE_IMPLEMENTATION
#include "stb_image_resize.h"

// TODO: When writing pngs, a profiler suggested that we spend a good chunk (~30%) of our time allocating
//       or reallocating memory. Maybe we could speed this up by allocating an arena ahead of time and
//       then sub-allocating from that, clearing it between files?
#define STB_IMAGE_WRITE_IMPLEMENTATION
#include "stb_image_write.h"

#include "tinydir.h"

typedef enum
{
    FT_INVALID,

    // Supported for input and output
    FT_BMP,
    FT_JPG,
    FT_PNG,
    FT_TGA,

    // Supported only for input
    FT_GIF,
    FT_HDR,
    FT_PIC,
    FT_PNM,
    FT_PSD,

} FileType;

const char* FileTypeExtension[] = {
    "INVALID",
    ".bmp",
    ".jpg",
    ".png",
    ".tga",
    ".gif",
    ".hdr",
    ".pic",
    ".pnm",
    ".psd"
};

#define MaxPathLen 1024
char* g_outputTypeStr = NULL;
char* g_outputPath = NULL;
int g_outputWidth = 0;
int g_outputHeight = 0;
double g_outputScale = 0.0;
int g_outputQuality = 0;

const char* getPathExtension(const char* path)
{
    int index = 0;
    while(path[index] != '\0') index++;
    while((index > 0) && (path[index] != '.')) index--;
    return &path[index];
}

const char* getPathFileName(const char* path)
{
    int index = 0;
    while(path[index] != '\0') index++;
    while((index > 0) && (path[index] != '/') && (path[index] != '\\')) index--;
    return &path[index]+1;
}

void processFile(const char* inputPath, const char* outputPathNoExt, FileType outputType, bool outputPathIsDir)
{
    const char* outputExt = FileTypeExtension[outputType];
    size_t outputExtLen = strlen(outputExt);

    size_t outPathNoExtLen = strlen(outputPathNoExt);
    char outputPath[MaxPathLen];
    outputPath[0] = 0;
    if(outputPathIsDir)
    {
        const char* inputFileName = getPathFileName(inputPath);
        const char* inputFileExt = getPathExtension(inputFileName);
        size_t inputFileNameLen = strlen(inputFileName);
        size_t inputFileExtLen = strlen(inputFileExt);

        if(outPathNoExtLen + 1 + inputFileNameLen - inputFileExtLen + outputExtLen >= MaxPathLen)
        {
            fprintf(stderr, "WARNING: Output path for input file %s is too long. Skipping...\n", inputPath);
            return;
        }
        strcpy(outputPath, outputPathNoExt);
        outputPath[outPathNoExtLen] = '/';
        strcpy(outputPath+outPathNoExtLen+1, inputFileName);
        strcpy(outputPath+outPathNoExtLen+1+inputFileNameLen-inputFileExtLen, outputExt);
    }
    else
    {
        if(outPathNoExtLen + outputExtLen >= MaxPathLen)
        {
            fprintf(stderr, "WARNING: Output path for input file %s is too long. Skipping...\n", inputPath);
            return;
        }

        strncpy(outputPath, outputPathNoExt, MaxPathLen);
        strcpy(outputPath+outPathNoExtLen, outputExt);
    }

    tinydir_file outputFileExistenceCheck;
    int outputFileExistence = tinydir_file_open(&outputFileExistenceCheck, outputPath);
    if(outputFileExistence >= 0)
    {
        fprintf(stderr, "WARNING: There is already a file at the output path: %s. Skipping the processing of %s to avoid data loss...\n", outputPath, inputPath);
        return;
    }

    int width;
    int height;
    int channels;
    uint8_t* data = stbi_load(inputPath, &width, &height, &channels, 0);
    if(data == NULL)
    {
        fprintf(stderr, "WARNING: Failed to read image input file %s. Skipping...\n", inputPath);
        return;
    }

    int resizedWidth = width;
    int resizedHeight = height;
    if(g_outputScale != 0.0)
    {
        resizedWidth = (int)(width*g_outputScale);
        resizedHeight = (int)(height*g_outputScale);
    }
    else
    {
        if(g_outputWidth != 0) resizedWidth = g_outputWidth;
        if(g_outputHeight != 0) resizedHeight = g_outputHeight;
    }

    if((resizedWidth != width) || (resizedHeight != height))
    {
        uint8_t* resizedData = (uint8_t*)malloc(resizedWidth*resizedHeight*channels);
        int resizeSuccess = stbir_resize_uint8(data, width, height, 0,
                                               resizedData, resizedWidth, resizedHeight, 0,
                                               channels);
        stbi_image_free(data);
        data = resizedData;
        width = resizedWidth;
        height = resizedHeight;

        if(resizeSuccess != 1)
        {
            fprintf(stderr, "WARNING: Failed to resize input image file %s. Skipping...\n", inputPath);
            return;
        }
    }

    int strideBytes = width*channels;
    int writeSuccess = 0;
    switch(outputType)
    {
        case FT_BMP: writeSuccess = stbi_write_bmp(outputPath, width, height, channels, data); break;
        case FT_JPG: writeSuccess = stbi_write_jpg(outputPath, width, height, channels, data, g_outputQuality); break;
        case FT_PNG: writeSuccess = stbi_write_png(outputPath, width, height, channels, data, strideBytes); break;
        case FT_TGA: writeSuccess = stbi_write_tga(outputPath, width, height, channels, data); break;
        default:
        {
            fprintf(stderr, "WARNING: Unexpected unrecognised output file type %d for input file %s. This is a bug.\n", outputType, inputPath);
        }
    }

    if(writeSuccess)
    {
        printf("Successfully processed %s -> %s\n", inputPath, outputPath);
    }
    else
    {
        fprintf(stderr, "WARNING: Failed to write image output file %s. Skipping...\n", outputPath);
    }

    stbi_image_free(data);
}

int main(int argc, char** argv)
{
    // TODO: Let users not pass in the type and instead of defaulting to png (or some other single type), we default to the type of the input image
    kgflags_string("type", "png", "The file type of the output images", false, &g_outputTypeStr);
    kgflags_string("output", ".", "The path of the output file(s). Can be a file name or a directory.", false, &g_outputPath);
    kgflags_int("width", 0, "The width to resize the output images to (unchanged if not specified)", false, &g_outputWidth);
    kgflags_int("height", 0, "The height to resize the output images to (unchanged if not specified)", false, &g_outputWidth);
    kgflags_double("scale", 0.0, "The factor by which to scale the input images", false, &g_outputScale);
    kgflags_int("quality", 100, "The quality of the output image encoding (0-100, must be supported by the format. Currently only jpg)", false, &g_outputQuality);
    // TODO: Let users pass in a prefix to add to the converted file name (default to "imged" or whatever)

    if(!kgflags_parse(argc, argv)) {
        kgflags_print_errors();
        kgflags_print_usage();
        return 1;
    }

    FileType outputType = FT_INVALID;
    if(strcmp(g_outputTypeStr, FileTypeExtension[FT_BMP]+1) == 0) outputType = FT_BMP;
    else if(strcmp(g_outputTypeStr, FileTypeExtension[FT_JPG]+1) == 0) outputType = FT_JPG;
    else if(strcmp(g_outputTypeStr, FileTypeExtension[FT_PNG]+1) == 0) outputType = FT_PNG;
    else if(strcmp(g_outputTypeStr, FileTypeExtension[FT_TGA]+1) == 0) outputType = FT_TGA;
    if(outputType == FT_INVALID)
    {
        fprintf(stderr, "ERROR: Invalid output type, supported extensions are: bmp, jpg, png, tga\n");
        return 1;
    }

    if(!g_outputTypeStr || (*g_outputTypeStr == 0))
    {
        fprintf(stderr, "ERROR: Invalid output path specified. Output path cannot be an empty string\n");
        return 1;
    }

    if((g_outputScale != 0.0) && ((g_outputWidth != 0) || (g_outputHeight != 0)))
    {
        fprintf(stderr, "ERROR: The scale parameter cannot be used in conjunction with the width or height parameter. Please either only use scale or only use width and/or height.\n");
        return 1;
    }

    bool outputIsDir = false;
    tinydir_file outputFile;
    int openOutputResult = tinydir_file_open(&outputFile, g_outputPath);
    if((openOutputResult >= 0) && outputFile.is_dir)
    {
        outputIsDir = true;
        size_t outputLen = strlen(g_outputPath);
        if((g_outputPath[outputLen-1] == '\\') || (g_outputPath[outputLen-1] == '/'))
        {
            g_outputPath[outputLen-1] = '\0';
        }
    }

    int inputSpecCount = kgflags_get_non_flag_args_count();
    for(int i=0; i<inputSpecCount; i++)
    {
        const char* inputSpec = kgflags_get_non_flag_arg(i);
        if(strcmp(inputSpec, "--help") == 0)
        {
            kgflags_print_usage();
            return 0;
        }
    }

    if(!outputIsDir && (inputSpecCount > 1))
    {
        fprintf(stderr, "ERROR: Output is not a directory and multiple input files were given. Either give a directory as output or only specify a single input file.\n");
        return 1;
    }

    // Actually process the files
    // TODO: I expect we could speed this up significantly if we ran multiple threads and
    //       split all the inputs across them. The main reason I haven't done it yet is just that
    //       the C stdlib doesn't have built-in thread support. Also if we do this we should check
    //       that we don't get bottlenecked by io/disk.
    for(int argIndex=0; argIndex<inputSpecCount; argIndex++)
    {
        const char* inputSpec = kgflags_get_non_flag_arg(argIndex);
        size_t wildCardIndex = 0;
        int wildCardCount = 0;
        for(size_t i=0; inputSpec[i] != '\0'; i++)
        {
            if(inputSpec[i] == '*')
            {
                wildCardCount++;
                wildCardIndex = i;
            }
        }

        if(wildCardCount > 1)
        {
            fprintf(stderr, "WARNING: Input '%s' contains multiple wildcard symbols. This is not currently supported. Skipping...\n", inputSpec);
            continue;
        }
        else if(wildCardCount == 1)
        {
            bool wildcardOnDirectories = false;
            for(size_t i=wildCardIndex+1; inputSpec[i] != '\0'; i++)
            {
                if((inputSpec[i] == '/') || (inputSpec[i] == '\\'))
                {
                    wildcardOnDirectories = true;
                    break;
                }
            }

            if(wildcardOnDirectories)
            {
                fprintf(stderr, "WARNING: Input '%s' contains wildcard symbols for non-leaf nodes of the directory tree. This is not currently supported. Skipping...\n", inputSpec);
                continue;
            }

            const char* inputFileName = getPathFileName(inputSpec);
            size_t inputFileNameLen = strlen(inputFileName);
            size_t pathChars = inputFileName - inputSpec;
            if(wildCardIndex < pathChars)
            {
                fprintf(stderr, "WARNING: Input '%s' appears to contain wildcard symbols in the name of a directory when it shouldn't. This is a bug. Skipping...\n", inputSpec);
                continue;
            }
            size_t fileNameWildCardIndex = wildCardIndex - pathChars;
            size_t prefixLen = fileNameWildCardIndex;
            size_t suffixLen = inputFileNameLen - (fileNameWildCardIndex + 1);

            char inputDirPath[MaxPathLen];
            if(inputFileName == inputSpec)
            {
                strcpy(inputDirPath, ".");
            }
            else
            {
                memcpy(inputDirPath, inputSpec, pathChars-1);
                inputDirPath[pathChars-1] = 0;
            }

            tinydir_dir inputDir;
            int dirOpenSuccess = tinydir_open(&inputDir, inputDirPath);
            if(dirOpenSuccess < 0)
            {
                fprintf(stderr, "WARNING: Failed to open input directory: %s. Skipping\n", inputDirPath);
                continue;
            }
            while(inputDir.has_next)
            {
                tinydir_file inputFileCandidate;
                tinydir_readfile(&inputDir, &inputFileCandidate);

                if((strcmp(inputFileCandidate.name, ".") == 0) || (strcmp(inputFileCandidate.name, "..") == 0))
                {
                    tinydir_next(&inputDir);
                    continue;
                }

                size_t candidateNameLen = strlen(inputFileCandidate.name);
                bool matches = true;
                if(candidateNameLen < prefixLen + suffixLen)
                {
                    matches = false;
                }
                for(int i=0; matches && i<prefixLen; i++)
                {
                    if(inputFileCandidate.name[i] != inputFileName[i])
                        matches = false;
                }
                for(int i=0; matches && i<suffixLen; i++)
                {
                    if(inputFileCandidate.name[candidateNameLen-i] != inputFileName[inputFileNameLen-i])
                        matches = false;
                }

                if(!matches)
                {
                    tinydir_next(&inputDir);
                    continue;
                }

                if(inputFileCandidate.is_dir)
                {
                    fprintf(stderr, "Input file %s matched the input pattern, but is a directory. Traversing directory trees is not currently supported. Skipping\n", inputFileCandidate.path);
                    tinydir_next(&inputDir);
                    continue;
                }

                processFile(inputFileCandidate.path, g_outputPath, outputType, outputIsDir);
                tinydir_next(&inputDir);
            }
            tinydir_close(&inputDir);
        }
        else // wildCardCount == 0
        {
            processFile(inputSpec, g_outputPath, outputType, outputIsDir);
        }

    }

    return 0;
}
