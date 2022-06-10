//
// Visual Studio Setup Script Generator
// 2020-08-28
// A simple tool to generate a batch file that sets all the environment variables required 
// to sensibly compile things with Visual Studio's C/C++ compiler.
//
// Heavily inspired by Ivan Yakymchak's QSetup (https://github.com/Yakvi/Qsetup). 
// We makes use of his (modified) copy of Jonathan Blow's microsoft_craziness.h, with our own 
// modification to also output the Visual Studio version.
//

#include "microsoft_craziness.h"

size_t printVarAssign(char* buffer, size_t bufferLength, const char* varName, const wchar_t* varValue)
{
    return snprintf(buffer, bufferLength, "SET %s=%ls;%%%s%%\n", varName, varValue, varName);
}

int main(int argc, char** argv)
{
    for(int i=0; i<argc; i++)
    {
        if((strcmp(argv[i], "--help") == 0) || (strcmp(argv[i], "-h") == 0))
        {
            printf("Visual Studio Setup Script Generator\n");
            printf("2020-08-28\n");
            printf("\n");
            printf("A simple tool to generate a batch file that sets all the environment variables required\n");
            printf("to sensibly compile things with Visual Studio's C/C++ compiler.");
            return 0;
        }
    }

    Find_Result result = find_visual_studio_and_windows_sdk();
    if(result.windows_sdk_version == 0)
    {
        free_resources(&result);
        fprintf(stderr, "ERROR: Unable to find visual studio!\n");
        return 1;
    }

    const size_t outCap = MAX_PATH*20;
    char* outStr = (char*)malloc(outCap);
    size_t outLen = 0;
    outLen += snprintf(outStr+outLen, outCap-outLen, "@echo off\n");
    outLen += snprintf(outStr+outLen, outCap-outLen, "SET VISUALSTUDIOVERSION=%ls\n", result.vs_version);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "Path", result.vs_exe_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "LIB", result.vs_library_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "LIB", result.windows_sdk_ucrt_library_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "LIB", result.windows_sdk_um_library_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "LIBPATH", result.windows_sdk_ucrt_library_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "LIBPATH", result.windows_sdk_um_library_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "INCLUDE", result.vs_include_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "INCLUDE", result.windows_sdk_ucrt_include_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "INCLUDE", result.windows_sdk_um_include_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "INCLUDE", result.windows_sdk_winrt_include_path);
    outLen += printVarAssign(outStr+outLen, outCap-outLen, "INCLUDE", result.windows_sdk_shared_include_path);
    outLen += snprintf(outStr+outLen, outCap-outLen, "echo Environment variables set. Windows SDK v%d, Visual Studio v%ls\n", result.windows_sdk_version, result.vs_version);
    free_resources(&result);

    const char* outputFileName = "vssetup.bat";
    FILE* outputFile = fopen(outputFileName, "wb");
    fprintf(outputFile, "%s", outStr);
    fclose(outputFile);
    printf("Environment variables written to %s.\n", outputFileName);

    free(outStr);
    return 0;
}

