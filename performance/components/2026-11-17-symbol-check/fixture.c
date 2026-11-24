static unsigned long long inner(unsigned long long a) { return a ^ 123; }
unsigned long long lustr_mixed_0123456789abcdef(unsigned long long a) { return inner(a); }
