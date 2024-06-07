#include "raylib.h"
#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <string.h>
#include <dirent.h>

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

int LoadPalette(const char *filepath, colour *colors, int *color_count) {
    FILE *file = fopen(filepath, "r");
    if (!file) {
        fprintf(stderr, "Could not open file: %s\n", filepath);
        return 1;
    }

    *color_count = 0;
    char hex_code[7];

    while (fscanf(file, "%6s", hex_code) == 1 && *color_count < 256) {
        unsigned int hex;
        sscanf(hex_code, "%x", &hex);
        colors[*color_count].r = (hex >> 16) & 0xFF;
        colors[*color_count].g = (hex >> 8) & 0xFF;
        colors[*color_count].b = hex & 0xFF;
        (*color_count)++;
    }
    fclose(file);
    return 0;
}

char** GetPaletteFiles(const char *directory, int *file_count) {
    DIR *dir = opendir(directory);
    struct dirent *entry;
    char **palette_files = NULL;
    *file_count = 0;

    if (dir) {
        while ((entry = readdir(dir)) != NULL) {
            if (strstr(entry->d_name, ".hex") != NULL) {
                palette_files = realloc(palette_files, (*file_count + 1) * sizeof(char*));
                if (palette_files == NULL) {
                    fprintf(stderr, "Memory allocation failed\n");
                    closedir(dir);
                    exit(EXIT_FAILURE);
                }
                size_t len = strlen(directory) + strlen(entry->d_name) + 2; // +2 for '/' and '\0'
                palette_files[*file_count] = malloc(len);
                if (palette_files[*file_count] == NULL) {
                    fprintf(stderr, "Memory allocation failed\n");
                    closedir(dir);
                    exit(EXIT_FAILURE);
                }
                snprintf(palette_files[*file_count], len, "%s/%s", directory, entry->d_name);
                (*file_count)++;
            }
        }
        closedir(dir);
    }
    return palette_files;
}

void FreePaletteFiles(char **palette_files, int file_count) {
    for (int i = 0; i < file_count; i++) {
        free(palette_files[i]);
    }
    free(palette_files);
}

const char* GetPalFileName(const char* path) {
    const char* filename = strrchr(path, '/');
    return filename ? filename + 1 : path;
}

int main(int argc, char *argv[]) {
    const char *directory = "../../palettes";
    int file_count = 0;
    char **palette_files = GetPaletteFiles(directory, &file_count);
    int current_palette_index = 0;

    if (file_count == 0) {
        fprintf(stderr, "No .hex files found in directory: %s\n", directory);
        return 1;
    }

    const char *filename = (argc == 2) ? argv[1] : palette_files[0];

    if (argc == 2) {
        FILE *file = fopen(filename, "r");
        if (!file) {
            fprintf(stderr, "Provided file does not exist: %s\n", argv[1]);
            FreePaletteFiles(palette_files, file_count);
            return 1;
        }
        fclose(file);
    } else {
        filename = palette_files[0];
    }

    InitWindow(800, 600, "PView");
    SetWindowState(FLAG_WINDOW_RESIZABLE);
    SetTargetFPS(60);

    colour colors[256];
    Color bgColour = RAYWHITE;
    Color textColour = BLACK;
    int color_count = 0;

    if (LoadPalette(filename, colors, &color_count)) {
        CloseWindow();
        FreePaletteFiles(palette_files, file_count);
        return 1;
    }

    while (!WindowShouldClose()) {
        if (IsKeyPressed(KEY_RIGHT)) {
            current_palette_index = (current_palette_index + 1) % file_count;
            filename = palette_files[current_palette_index];
            if (LoadPalette(filename, colors, &color_count)) {
                CloseWindow();
                FreePaletteFiles(palette_files, file_count);
                return 1;
            }
        }

        if (IsKeyPressed(KEY_LEFT)) {
            current_palette_index = (current_palette_index - 1 + file_count) % file_count;
            filename = palette_files[current_palette_index];
            if (LoadPalette(filename, colors, &color_count)) {
                CloseWindow();
                FreePaletteFiles(palette_files, file_count);
                return 1;
            }
        }
        
        if (IsKeyPressed(KEY_SPACE)) {
            if (bgColour.r == RAYWHITE.r && bgColour.g == RAYWHITE.g && bgColour.b == RAYWHITE.b && bgColour.a == RAYWHITE.a) {
                bgColour = BLACK;
                textColour = RAYWHITE;
            } else {
                bgColour = RAYWHITE;
                textColour = BLACK;
            }
        }

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
        ClearBackground(bgColour);

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

        const char *display_filename = GetFileName(filename);
        const char *navigation_text = "<-- Prev Next -->";

        Vector2 countSize = MeasureTextEx(GetFontDefault(), count_text, 20, 1);
        Vector2 navigationSize = MeasureTextEx(GetFontDefault(), navigation_text, 20, 1);
        Vector2 filenameSize = MeasureTextEx(GetFontDefault(), display_filename, 20, 1);

        int count_x = MARGIN;

        int filename_x = windowWidth - filenameSize.x - MARGIN;

        int nav_x = (count_x + countSize.x + filename_x) / 2 - navigationSize.x / 2;

        int text_y = windowHeight - 30;

        // Draw the texts
        DrawText(count_text, count_x, text_y, 20, textColour);
        DrawText(navigation_text, nav_x, text_y, 20, textColour);
        DrawText(display_filename, filename_x, text_y, 20, textColour);


        EndDrawing();
    }

    CloseWindow();
    FreePaletteFiles(palette_files, file_count);

    return 0;
}

