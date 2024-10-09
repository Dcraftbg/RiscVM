#include <stdarg.h>
#include <stddef.h>
#define SERIAL_OUT 0x6969
#define EXIT       0x7000
static void exit(char code) {
    *((char*)EXIT) = code;
}
static void serial_putc(char c) {
    *((char*)SERIAL_OUT) = c;
}
static void serial_puts(const char* str) {
    while(*str) {
        serial_putc(*(str++));
    }
}
static void utoa(char* buf, size_t len, unsigned int a) {
   
    while(a > 0 && len > 0) {
        serial_puts("Here\n");
        *(buf++) = (a % 10) + '0';
        a /= 10;
        len--;
    }
}
static void itoa(char* buf, size_t len, int a) {
    if(len == 0) return;
    if(a < 0) {
        *(buf++) = '-';
        len--;
        utoa(buf, len, -a);
    } else {
        utoa(buf, len, a);
    }
}
void printf(const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    while(*fmt) {
        char c = *fmt++;
        switch(c) {
        case '%': {
            switch(c=(*fmt++)) {
            case 'd': {
                char ibuf[22]={0};
                itoa(ibuf, sizeof(ibuf)-1, va_arg(args, int));
                serial_puts(ibuf);
            } break;
            case '%':
                serial_putc('%');
                break;
            default:
                serial_putc('%');
                serial_putc(c);
            }
        } break;
        default:
            serial_putc(c);
        }
    }
    va_end(args);
}
void _start()  __attribute__((section(".entry")));
void _start() {
    serial_puts("Hello World!\n");
    printf("Test %d\n", 1234);
    exit(1);
    for(;;);
}
