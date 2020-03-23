struct COFFHeader 
{
    short Machine;
    short NumberOfSections;
    long TimeDateStamp;
    long PointerToSymbolTable;
    unsigned long long int NumberOfSymbols;
    short SizeOfOptionalHeader;
    short Characteristics;
};