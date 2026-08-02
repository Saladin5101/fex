# FEX - Format EXchange
Welcome.<br>
Do you want chanage the format of binary files ? Do you have a new format but no one tool can make it ? I have this problem , too. What do we need ? We need **A easy-to-use and useful tool** to do that , so , I create FEX.
## Quick Start
### 1. Install
We only support FEX in Linux(I am using Fedora 43 Workstation) and macOS(I am using macOS 26), **WE DO NOY SUPPORT WINDOWS** , if you want to use FEX , you can use WSL.<br>
You should install it due Cargo , command is:
```bash
cargo install fex-tool
```
Then Cargo will do anything for install , you only need to waitting for install end<br>
### 2. You Have To Know Before Install 
#### 2.a Installation Prerequisites
- 1. First , you should install Cargo and Rust tools before you install , use ```curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh```
- 2. Then , you have to know your Rust version , you have to use latest version to install ;
- 3. Finally ， please check do you have 2 GiB storage space (Note:FEX don't need too more , teoretically, FEX may only require a few hundred MiB or less. Asking you to reserve so much space is to prevent issues caused by not cleaning temporary files for a long time (although FEX won't generate that much, who can say for sure? I recently ran into a storage overflow problem, where Docker, for some reason, used 257GB on my 256GB Mac).) , 2 GiB RAM to make install done.
#### 2.b Errors You May Encounter During Installation
- 1. If you encounter an error during installation, please check your network connection first, as Cargo needs to download dependencies from the internet;
- 2. If you encounter permission denied errors, please check if you have the necessary permissions to install software on your system;
- 3. If GNU Make reports an error , please try build from source code , if you continue to see that please send emaail to <saladin131211@gmail.com> or <saladin510@outlook.com>.(*If I don't even know how to solve it, then you're on your own.*)
### 3. Usage
#### 3.a Command Line (Shell)
You can use FEX in command line , the syntax is :
```bash
fex -f <Old Format> -t <New Format> -i <Input> -o <Output> <Other command>
```
For example :
```bash
fex -f pe -t elf -i input.exe -o ouput --do-not-keep-old-file
```
This command will convert input.exe from PE format to ELF format and remove old file .<br>
#### 3.b Config File
You can create a JSON file , it likes :
```json
{
  "name": "ELF32",
  "description": "Executable and Linkable Format (32-bit)",
  "author": "Unix System Laboratories",
  "version": "1.0",
  "endianness": "little",
  "magic": [127, 69, 76, 70],
  "header_size": 52,
  "header_fields": [
    {
      "name": "e_ident",
      "offset": 0,
      "size": 16,
      "type": "bytes",
      "description": "ELF identification",
      "values": {
        "magic": [127, 69, 76, 70],
        "class": 1,
        "data": 1,
        "version": 1,
        "osabi": 0,
        "abiversion": 0
      }
    },
    {
      "name": "e_type",
      "offset": 16,
      "size": 2,
      "type": "u16",
      "description": "Object file type",
      "enum": {
        "0": "ET_NONE",
        "1": "ET_REL",
        "2": "ET_EXEC",
        "3": "ET_DYN",
        "4": "ET_CORE"
      }
    },
    {
      "name": "e_machine",
      "offset": 18,
      "size": 2,
      "type": "u16",
      "description": "Architecture",
      "enum": {
        "0x00": "No specific instruction set",
        "0x02": "SPARC",
        "0x03": "x86",
        "0x08": "MIPS",
        "0x14": "PowerPC",
        "0x28": "ARM",
        "0x2A": "SuperH",
        "0x32": "IA-64",
        "0x3E": "x86-64",
        "0xB7": "AArch64",
        "0xF3": "RISC-V"
      }
    },
    {
      "name": "e_version",
      "offset": 20,
      "size": 4,
      "type": "u32",
      "description": "Object file version",
      "default": 1
    },
    {
      "name": "e_entry",
      "offset": 24,
      "size": 4,
      "type": "u32",
      "description": "Entry point virtual address"
    },
    {
      "name": "e_phoff",
      "offset": 28,
      "size": 4,
      "type": "u32",
      "description": "Program header table file offset"
    },
    {
      "name": "e_shoff",
      "offset": 32,
      "size": 4,
      "type": "u32",
      "description": "Section header table file offset"
    },
    {
      "name": "e_flags",
      "offset": 36,
      "size": 4,
      "type": "u32",
      "description": "Processor-specific flags"
    },
    {
      "name": "e_ehsize",
      "offset": 40,
      "size": 2,
      "type": "u16",
      "description": "ELF header size in bytes",
      "default": 52
    },
    {
      "name": "e_phentsize",
      "offset": 42,
      "size": 2,
      "type": "u16",
      "description": "Program header table entry size"
    },
    {
      "name": "e_phnum",
      "offset": 44,
      "size": 2,
      "type": "u16",
      "description": "Program header table entry count"
    },
    {
      "name": "e_shentsize",
      "offset": 46,
      "size": 2,
      "type": "u16",
      "description": "Section header table entry size"
    },
    {
      "name": "e_shnum",
      "offset": 48,
      "size": 2,
      "type": "u16",
      "description": "Section header table entry count"
    },
    {
      "name": "e_shstrndx",
      "offset": 50,
      "size": 2,
      "type": "u16",
      "description": "Section header string table index"
    }
  ],
  "sections": [
    {
      "name": ".text",
      "type": "code",
      "offset_expr": "e_phoff + e_phnum * e_phentsize",
      "size_expr": "file_size - (e_phoff + e_phnum * e_phentsize)",
      "flags": ["executable", "readonly"],
      "description": "Executable code"
    },
    {
      "name": ".data",
      "type": "data",
      "offset_expr": "e_shoff + e_shnum * e_shentsize",
      "size_expr": "file_size - (e_shoff + e_shnum * e_shentsize)",
      "flags": ["writable", "readonly"],
      "description": "Initialized data"
    }
  ],
  "validators": [
    {
      "type": "magic_check",
      "offset": 0,
      "bytes": [127, 69, 76, 70]
    },
    {
      "type": "size_check",
      "field": "e_ehsize",
      "value": 52
    }
  ],
  "transformers": {
    "to_flat": {
      "description": "Extract .text section",
      "sections": [".text"],
      "strip_headers": true
    },
    "from_flat": {
      "description": "Wrap raw code in ELF header",
      "template": "elf_header.bin"
    }
  }
}
```
Then , you can use command :
```bash
fex config --use <Your JSON File Name> -i <Old File> -o <New File> --it-is-convert / --it-is-converted
```
**Note:If your config file is for convert a file to the format specified in your config file , use ```--it-is-convert``` , or , if your config file is specified a From Format , use ```--it-is-converted```.**<br>
**Note: If you use the wrong --it-is-convert and --it-is-converted parameters... well, then you'll have to debug it yourself. Good luck chatting with your boss?**<br>
For example :
```bash
fex config --use test.json -i input.input -o output.exe --it-is-converted
```
However, I don't really recommend this approach, unless you have developed a completely new format yourself. Otherwise, debugging the format of your configuration file or errors could take several times more than your actual working time. If, like me, you are an independent developer, this might not be a big deal. But if you are an employee... I can't imagine how you would explain to your boss that you need to spend a significant portion of your work time debugging a configuration file that seems completely useless.<br>
If you want to change a file format to a new format but FEX don't have this format's formula ， please upload your json file if your boss is doesn't know , 'cause our formula is use json file to make ...<br>
For the binary file formats of formulae that we support for input or output, please refer to the next chapter.
## Supported Formats
We supported these formats to input when you don't use config file:
- ELF(UNIX & Linux)
- PE(Windows)
- Mach-O(macOS)
- Intel HEX
- BIN file

## Contact
If you have any questions or suggestions , please send email to <saladin131211@gmail.com> or <saladin510@outlook.com><br>
If you found out some issues or bugs , please open a issue<br>
If you want add a new formula , please use "make add-formula"<br>
If you want contribute code , please fork this repo and make a pull request<br>
## License
This project is licensed under the GNU GPL - See COPYING(COPYING) to study more.
