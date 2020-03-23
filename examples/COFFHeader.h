struct COFFHeader 
{
    short Machine;
    short NumberOfSections;
    long TimeDateStamp;
    long PointerToSymbolTable;
    int NumberOfSymbols;
    short SizeOfOptionalHeader;
    short Characteristics;
};