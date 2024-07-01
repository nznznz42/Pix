# Workflow Files

## Idea

So the basic idea here is to find a way to allow the user to give the program a json file that describes a sequence of operations to be run on the image

## Format

```json
{
  "FilePath": "Path/To/File",
  "OutputFilePath": "Path/To/OutputFile",
  
  "Instructions": ["OPERATIONS"]
}

```

This seems like a reasonably simple initial format to start with, the general idea here is to be able to be able to describe any possible operation with all of its dependencies as a single unit.

### Instruction format

```json
{
  "InstructionName": "NameOfInstruction",
  "Args": []
}
```

## Possible Operations

this list is not exhaustive and can be changed in the future:

- loadImage(filepath: &str)
- saveImage(filepath: &str)
- createPalette(outputfilepath: &str, name: &str, n_colours: u32, strategy: SelectionStrategy)
- loadPalette(filepath: &str)
- savePalette(outputfilepath: &str)
- createMask(maskid: u32, legalcolours: Vec<Rgb>)
- combineMasks(maskid: u32, maskids: Vec<u32>)
- applyMask(maskids: Vec<u32>)
- applyDither(mode: DitherMode)
- applyPixelation(pxfactor: u32)

## Program Flow

1. Invoke CLI with path to workflow file
2. Deserialize into Job Object 
3. Load Image into memory 
4. For each Instruction in queue job.execute(Instruction) (This uses the in-memory image exclusively)
5. Save Image
6. Exit

### Job Format

```rust

struct Job {
    workflowPath: String,
    image: Image,
    instructions: Vec<Instructions>
}
```

This is roughly what the job object should look like, any associated functions should be related to reading, parsing and executing a workflow file

### Instruction Set

Might be useful to create a list of primitive atomic operations that can be combined to make more ergonomic instructions that can do more things?