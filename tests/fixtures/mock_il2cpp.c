#include <stdint.h>

// Mock structures matching IL2CPP version 29.0.
typedef struct {
	uint64_t methodPointersCount;
	void** methodPointers;
	void* genericMethodPointers;
	void* invokerPointers;
	void* customAttributeGenerators;
	void* unresolvedVirtualCallPointers;
	void* interopData;
	void* windowsRuntimeFactoryTable;
	void* codeGenModulesCount; // This is actually a uint32_t but aligned to ptr_size.
	void* codeGenModules;
} Il2CppCodeRegistration;

// 1. Feature byte string search target (dllva).
const char mscorlib_name[] = "mscorlib.dll";

// 2. mscorlib_image points to mscorlib_name (refva points to dllva).
const char* mscorlib_image = mscorlib_name;

// 3. g_CodeGenModules contains mscorlib_image (refva2 points to refva).
const char** g_CodeGenModules[] = { &mscorlib_image };

// Mock methods.
void mock_method_1(void) {}
void mock_method_2(void) {}
void* mock_methods[] = { &mock_method_1, &mock_method_2 };

// 4. Code Registration structure.
// In IL2CPP v29, codeGenModules is at offset 14 * ptr_size (112 bytes for 64-bit).
// So we pad it out with dummy fields.
typedef struct {
	void* dummy[13];
	uint64_t codeGenModulesCount; // Offset 13 * ptr_size (contains image_count = 1)
	void* codeGenModules;         // Offset 14 * ptr_size (this is refva3!)
} CodeRegHeuristic_v29;

CodeRegHeuristic_v29 g_CodeRegistration = {
	.dummy = {
		&mock_methods, // methodPointers
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
	},
	.codeGenModulesCount = 1, // image_count = 1
	.codeGenModules = &g_CodeGenModules
};

// Full Metadata Registration structure.
typedef struct {
	uint64_t genericClassesCount;          // Offset 0
	void* genericClasses;                  // Offset 1
	uint64_t genericInstsCount;            // Offset 2
	void* genericInsts;                    // Offset 3
	uint64_t genericMethodTableCount;      // Offset 4
	void* genericMethodTable;              // Offset 5
	uint64_t typesCount;                   // Offset 6
	void* types;                           // Offset 7
	uint64_t methodSpecsCount;             // Offset 8
	void* methodSpecs;                     // Offset 9
	uint64_t fieldOffsetsCount;            // Offset 10
	void* fieldOffsets;                    // Offset 11
	uint64_t typeDefinitionsSizesCount;    // Offset 12
	void* typeDefinitionsSizes;            // Offset 13
	uint64_t metadataUsagesCount;          // Offset 14
	void* metadataUsages;                  // Offset 15
} Il2CppMetadataRegistration;

// Dummy types pointers in data section.
typedef struct {
	void* datapoint;
	uint32_t bits;
} MockIl2CppType;

MockIl2CppType dummy_type_1 = {
	.datapoint = (void*)0,
	.bits = (1 << 16) // ty = 1 (Void)
};
MockIl2CppType dummy_type_2 = {
	.datapoint = (void*)0,
	.bits = (1 << 16) // ty = 1 (Void)
};

MockIl2CppType* dummy_types[] = { &dummy_type_1, &dummy_type_2 };

// Dummy field offsets.
uint32_t dummy_field_offset_1 = 0;
uint32_t dummy_field_offset_2 = 0;
uint32_t* dummy_field_offsets[] = { &dummy_field_offset_1, &dummy_field_offset_2 };

// Dummy type definition sizes.
uint32_t dummy_type_def_sizes[] = { 0, 0 };

Il2CppMetadataRegistration g_MetadataRegistration = {
	.genericClassesCount = 0,
	.genericClasses = 0,
	.genericInstsCount = 0,
	.genericInsts = 0,
	.genericMethodTableCount = 0,
	.genericMethodTable = 0,
	.typesCount = 2,
	.types = &dummy_types,
	.methodSpecsCount = 0,
	.methodSpecs = 0,
	.fieldOffsetsCount = 2,
	.fieldOffsets = &dummy_field_offsets,
	.typeDefinitionsSizesCount = 2,
	.typeDefinitionsSizes = &dummy_types,
	.metadataUsagesCount = 0,
	.metadataUsages = 0
};

// Exported symbols to ensure they are referenced and compiled into data section.
void* dummy_exports[] = {
	&g_CodeRegistration,
	&g_MetadataRegistration,
	&mscorlib_name
};

int main(void) {
	return 0;
}
