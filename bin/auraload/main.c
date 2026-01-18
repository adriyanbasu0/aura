#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <sys/mman.h>
#include <unistd.h>
#include <fcntl.h>

#define AURA_MAGIC 0x41555241
#define AURA_VERSION 1

typedef struct {
    uint8_t magic[4];
    uint8_t version;
    uint8_t flags;
    uint16_t reserved;
    uint64_t entry_point;
    uint64_t stack_size;
    uint64_t text_offset;
    uint64_t text_size;
    uint64_t data_offset;
    uint64_t data_size;
    uint64_t bss_size;
    uint64_t reloc_count;
    uint64_t symbol_count;
} AuraHeader;

extern void trampoline(void *entry, void *stack);

static void *align_up(void *ptr, size_t alignment) {
    uintptr_t addr = (uintptr_t)ptr;
    addr = (addr + alignment - 1) & ~(alignment - 1);
    return (void *)addr;
}

int main(int argc, char **argv) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <program.aura>\n", argv[0]);
        return 1;
    }

    const char *filename = argv[1];
    int fd = open(filename, O_RDONLY);
    if (fd < 0) {
        fprintf(stderr, "Error: Cannot open file: %s\n", filename);
        return 1;
    }

    off_t file_size = lseek(fd, 0, SEEK_END);
    if (file_size < 0) {
        fprintf(stderr, "Error: Cannot get file size\n");
        close(fd);
        return 1;
    }

    AuraHeader header;
    if (pread(fd, &header, sizeof(header), 0) != sizeof(header)) {
        fprintf(stderr, "Error: Cannot read header\n");
        close(fd);
        return 1;
    }

    if (header.magic[0] != 'A' || header.magic[1] != 'U' ||
        header.magic[2] != 'R' || header.magic[3] != 'A') {
        fprintf(stderr, "Error: Invalid magic number\n");
        close(fd);
        return 1;
    }

    if (header.version != AURA_VERSION) {
        fprintf(stderr, "Error: Unsupported version: %u\n", header.version);
        close(fd);
        return 1;
    }

    size_t page_size = sysconf(_SC_PAGESIZE);
    size_t total_size = header.text_size + header.data_size + page_size;
    
    void *code_addr = mmap(NULL, total_size,
                           PROT_READ | PROT_WRITE | PROT_EXEC,
                           MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (code_addr == MAP_FAILED) {
        fprintf(stderr, "Error: Cannot allocate memory\n");
        close(fd);
        return 1;
    }

    memset(code_addr, 0, total_size);

    void *aligned_text = align_up(code_addr, page_size);
    void *aligned_data = (void *)((uintptr_t)aligned_text + header.text_size);

    if (pread(fd, aligned_text, header.text_size, header.text_offset)
        != (ssize_t)header.text_size) {
        fprintf(stderr, "Error: Cannot read text section\n");
        munmap(code_addr, total_size);
        close(fd);
        return 1;
    }

    if (header.data_size > 0) {
        if (pread(fd, aligned_data, header.data_size, header.data_offset)
            != (ssize_t)header.data_size) {
            fprintf(stderr, "Error: Cannot read data section\n");
            munmap(code_addr, total_size);
            close(fd);
            return 1;
        }
    }

    uint64_t data_base_addr = 0x1000000;
    void *data_addr = mmap((void *)data_base_addr, header.data_size + page_size,
                            PROT_READ | PROT_WRITE,
                            MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED, -1, 0);
    if (data_addr != (void *)data_base_addr && header.data_size > 0) {
        fprintf(stderr, "Error: Cannot map data at fixed address\n");
        munmap(code_addr, total_size);
        close(fd);
        return 1;
    }

    if (header.data_size > 0) {
        memcpy(data_addr, aligned_data, header.data_size);
    }

    void *stack_addr = mmap(NULL, header.stack_size + page_size,
                            PROT_READ | PROT_WRITE,
                            MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
    if (stack_addr == MAP_FAILED) {
        fprintf(stderr, "Error: Cannot allocate stack\n");
        munmap(code_addr, total_size);
        if (header.data_size > 0) {
            munmap(data_addr, header.data_size + page_size);
        }
        close(fd);
        return 1;
    }

    void *aligned_stack = align_up(stack_addr + header.stack_size, 16);

    close(fd);

    void *entry_point = (void *)((uintptr_t)aligned_text + header.entry_point);
    trampoline(entry_point, aligned_stack);

    return 0;
}
