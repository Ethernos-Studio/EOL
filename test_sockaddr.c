#include <stdio.h>
#include <winsock2.h>

int main() {
    WSADATA wsaData;
    WSAStartup(MAKEWORD(2, 2), &wsaData);
    
    struct sockaddr_in addr;
    addr.sin_family = AF_INET;
    addr.sin_port = htons(10808);
    addr.sin_addr.s_addr = INADDR_ANY;
    
    printf("sizeof(sockaddr_in) = %zu\n", sizeof(struct sockaddr_in));
    printf("sin_family at offset 0, size %zu\n", sizeof(addr.sin_family));
    printf("sin_port at offset 2, size %zu\n", sizeof(addr.sin_port));
    printf("sin_addr at offset 4, size %zu\n", sizeof(addr.sin_addr));
    
    printf("\nMemory layout (hex):\n");
    unsigned char *p = (unsigned char *)&addr;
    for (int i = 0; i < 16; i++) {
        printf("%02X ", p[i]);
        if ((i + 1) % 4 == 0) printf("\n");
    }
    
    printf("\nExpected for port 10808 (0x2A38 in host order):\n");
    printf("Family: 02 00 (AF_INET in little-endian)\n");
    printf("Port:   38 2A (10808 in network/big-endian order)\n");
    printf("Addr:   00 00 00 00 (INADDR_ANY)\n");
    
    // Test bind
    SOCKET sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    int result = bind(sock, (struct sockaddr *)&addr, sizeof(addr));
    printf("\nBind result: %d\n", result);
    if (result != 0) {
        printf("Error: %d\n", WSAGetLastError());
    }
    
    closesocket(sock);
    WSACleanup();
    return 0;
}
