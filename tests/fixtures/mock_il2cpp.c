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

// Mock methods.
void mock_method_1(void) {}
void mock_method_2(void) {}
void* mock_methods[] = { &mock_method_1, &mock_method_2 };

// 1. Feature byte string search target (dllva).
const char mscorlib_name[] = "mscorlib.dll";

// 2. Real code gen module struct.
typedef struct {
	const char* moduleName;
	int64_t methodPointerCount;
	void** methodPointers;
	int64_t adjustorThunkCount;
	void* adjustorThunks;
	void* invokerIndices;
	uint64_t reversePinvokeWrapperCount;
	void* reversePinvokeWrapperIndices;
	int64_t rgctxRangesCount;
	void* rgctxRanges;
	int64_t rgctxsCount;
	void* rgctxs;
	void* debuggerMetadata;
	void* moduleInitializer;
	void* staticConstructorTypeIndices;
	void* metadataRegistration;
	void* codeRegistration;
} Il2CppCodeGenModule;

const Il2CppCodeGenModule g_mscorlib_CodeGenModule = {
	.moduleName = mscorlib_name,
	.methodPointerCount = 2,
	.methodPointers = mock_methods,
	.adjustorThunkCount = 0,
	.adjustorThunks = 0,
	.invokerIndices = 0,
	.reversePinvokeWrapperCount = 0,
	.reversePinvokeWrapperIndices = 0,
	.rgctxRangesCount = 0,
	.rgctxRanges = 0,
	.rgctxsCount = 0,
	.rgctxs = 0,
	.debuggerMetadata = 0,
	.moduleInitializer = 0,
	.staticConstructorTypeIndices = 0,
	.metadataRegistration = 0,
	.codeRegistration = 0
};

// 3. g_CodeGenModules contains a pointer to g_mscorlib_CodeGenModule (refva2 points to refva).
const Il2CppCodeGenModule* g_CodeGenModules[] = { &g_mscorlib_CodeGenModule };

// 4. Code registration structure.
// In IL2CPP v29, codeGenModules is at offset 14 * ptr_size (112 bytes for 64-bit).
// So we pad it out with dummy fields.
typedef struct {
	void* dummy[13];
	uint64_t codeGenModulesCount; // Offset 13 * ptr_size (contains image_count = 1).
	void* codeGenModules;         // Offset 14 * ptr_size (this is refva3!).
} CodeRegHeuristic_v29;

CodeRegHeuristic_v29 g_CodeRegistration = {
	.dummy = {
		&mock_methods, // Method pointers.
		0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
	},
	.codeGenModulesCount = 1, // image_count = 1.
	.codeGenModules = &g_CodeGenModules
};

// Full metadata registration structure.
typedef struct {
	uint64_t genericClassesCount;          // Offset 0.
	void* genericClasses;                  // Offset 1.
	uint64_t genericInstsCount;            // Offset 2.
	void* genericInsts;                    // Offset 3.
	uint64_t genericMethodTableCount;      // Offset 4.
	void* genericMethodTable;              // Offset 5.
	uint64_t typesCount;                   // Offset 6.
	void* types;                           // Offset 7.
	uint64_t methodSpecsCount;             // Offset 8.
	void* methodSpecs;                     // Offset 9.
	uint64_t fieldOffsetsCount;            // Offset 10.
	void* fieldOffsets;                    // Offset 11.
	uint64_t typeDefinitionsSizesCount;    // Offset 12.
	void* typeDefinitionsSizes;            // Offset 13.
	uint64_t metadataUsagesCount;          // Offset 14.
	void* metadataUsages;                  // Offset 15.
} Il2CppMetadataRegistration;

// Dummy types pointers in the data section.
typedef struct {
	void* datapoint;
	uint32_t bits;
} MockIl2CppType;

MockIl2CppType dummy_type_1 = {
	.datapoint = (void*)0,
	.bits = (1 << 16) // ty = 1 (void).
};
MockIl2CppType dummy_type_2 = {
	.datapoint = (void*)0,
	.bits = (1 << 16) // ty = 1 (void).
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

// Exported symbols to ensure they are referenced and compiled into the data section.
void* dummy_exports[] = {
	&g_CodeRegistration,
	&g_MetadataRegistration,
	&mscorlib_name
};

int main(void) {
	return 0;
}
