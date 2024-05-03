#include <stdio.h>
#include <stdint.h>

extern int64_t print(int64_t);
extern int64_t printchar(int64_t);
extern int64_t println(int64_t);
extern int64_t printcharln(int64_t);

int64_t print(int64_t c) {
	printf("%d", c);
	return c;
}
int64_t println(int64_t c) {
	printf("%d\n", c);
	return c;
}

int64_t printchar(int64_t c) {
	printf("%c", (char)c);
	return c;
}
int64_t printcharln(int64_t c) {
	printf("%c\n", (char)c);
	return c;
}

int64_t readchar() {
	char choice;
	scanf(" %c", &choice);	
	return (int64_t)choice;
}
