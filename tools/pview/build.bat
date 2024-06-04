@echo off
REM feel free to change this to whatever fits your needs
set INCLUDE_DIR=.\include
set LIB_DIR=.\lib
set OUTPUT_EXE=pview.exe
set SOURCE_FILE=pview.c

gcc -std=c11 -I%INCLUDE_DIR% -o %OUTPUT_EXE% %SOURCE_FILE% -L%LIB_DIR% -lraylib -lglfw3 -lopengl32 -lgdi32 -lwinmm -mwindows

REM if this compiles properly remove the pause so it closes the window next time
pause

