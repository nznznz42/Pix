#include "raylib.h"
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <string.h>

#define MARGIN 10
#define BOTTOM_MARGIN 40

typedef struct {
    unsigned char r, g, b;
} colour; 

float CalculateLuminance(Color color) {
    return (0.299 * color.r + 0.587 * color.g + 0.114 * color.b) / 255.0;
}

Color GetContrastingTextColor(Color bgColor) {
    return (CalculateLuminance(bgColor) > 0.5) ? BLACK : WHITE;
}

void CalculateIdealSquareSize(int n, int x, int y, int *rows, int *cols, int *cellSize) {
    float ratio = (float)x / y;
    float ncols_float = sqrtf(n * ratio);
    float nrows_float = n / ncols_float;

    int nrows1 = ceilf(nrows_float);
    int ncols1 = ceilf(n / (float)nrows1);
    while (nrows1 * ratio < ncols1) {
        nrows1++;
        ncols1 = ceilf(n / (float)nrows1);
    }
    float cell_size1 = (float)y / nrows1;

    int ncols2 = ceilf(ncols_float);
    int nrows2 = ceilf(n / (float)ncols2);
    while (ncols2 < nrows2 * ratio) {
        ncols2++;
        nrows2 = ceilf(n / (float)ncols2);
    }
    float cell_size2 = (float)x / ncols2;

    if (cell_size1 < cell_size2) {
        *rows = nrows2;
        *cols = ncols2;
        *cellSize = (int)cell_size2;
    } else {
        *rows = nrows1;
        *cols = ncols1;
        *cellSize = (int)cell_size1;
    }
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <palette.hex>\n", argv[0]);
        return 1;
    }

    char *filename = strrchr(argv[1], '/');
    filename = (filename != NULL) ? filename + 1 : argv[1];

    InitWindow(800, 600, "PView");
    SetWindowState(FLAG_WINDOW_RESIZABLE);
    SetTargetFPS(60);

    FILE *file = fopen(argv[1], "r");
    if (!file) {
        fprintf(stderr, "Could not open file: %s\n", argv[1]);
        CloseWindow();
        return 1;
    }
    
/* might want to modify this to support an arbitrary number of colours at some point 
 * things to change:
 * 1. make colors a vector
 * 2. revamp the text sizing logic to just not print it if it gets too small
 * 3. add in hue shifting or whatever palette editing features one might want
 * 4. clean up code so it isn't a disgusting throwaway script like it is right now
*/
    colour colors[256];
    int color_count = 0;
    char hex_code[7];

    while (fscanf(file, "%6s", hex_code) == 1 && color_count < 256) {
        unsigned int hex;
        sscanf(hex_code, "%x", &hex);
        colors[color_count].r = (hex >> 16) & 0xFF;
        colors[color_count].g = (hex >> 8) & 0xFF;
        colors[color_count].b = hex & 0xFF;
        color_count++;
    }
    fclose(file);

    while (!WindowShouldClose()) {
        int windowWidth = GetScreenWidth();
        int windowHeight = GetScreenHeight();

        int rows, cols, cellSize;
        CalculateIdealSquareSize(color_count, windowWidth - 2 * MARGIN, windowHeight - BOTTOM_MARGIN - MARGIN, &rows, &cols, &cellSize);

        if (cols * (cellSize + MARGIN) - MARGIN > windowWidth - 2 * MARGIN || rows * (cellSize + MARGIN) - MARGIN > windowHeight - BOTTOM_MARGIN - MARGIN) {
            float aspectRatio = (float)(windowWidth - 2 * MARGIN) / (windowHeight - BOTTOM_MARGIN - MARGIN);
            cols = ceil(sqrt(color_count * aspectRatio));
            rows = ceil((float)color_count / cols);
            cellSize = fmin((windowWidth - 2 * MARGIN) / cols, (windowHeight - BOTTOM_MARGIN - MARGIN) / rows) - MARGIN;
        }

        BeginDrawing();
        ClearBackground(RAYWHITE);

        for (int i = 0; i < color_count; i++) {
            int row = i / cols;
            int col = i % cols;

            int x = col * (cellSize + MARGIN) + MARGIN;
            int y = row * (cellSize + MARGIN) + MARGIN;

            Color c = { colors[i].r, colors[i].g, colors[i].b, 255 }; 
            DrawRectangle(x, y, cellSize, cellSize, c);

            char text[8];
            snprintf(text, sizeof(text), "%02X%02X%02X", colors[i].r, colors[i].g, colors[i].b);
            Vector2 textSize = MeasureTextEx(GetFontDefault(), text, 12, 1);
            Color textColor = GetContrastingTextColor(c);
            DrawTextEx(GetFontDefault(), text, (Vector2){ x + (cellSize - textSize.x) / 2, y + (cellSize - textSize.y) / 2 }, 12, 1, textColor);
        }

        char count_text[50];
        snprintf(count_text, sizeof(count_text), "Total Colors: %d", color_count);
        DrawText(count_text, MARGIN, windowHeight - 30, 20, BLACK);

        Vector2 filenameSize = MeasureTextEx(GetFontDefault(), filename, 20, 1);
        DrawText(filename, windowWidth - filenameSize.x - MARGIN, windowHeight - 30, 20, BLACK);

        EndDrawing();
    }

    CloseWindow();

    return 0;
}

