struct binder_write_read {
    __u64 write_size;
    __u64 write_consumed;
    __u64 write_buffer;
    __u64 read_size;
    __u64 read_consumed;
    __u64 read_buffer;
};

// binder_size_t is __u64 for android > 4.4
struct binder_transaction_data {
    union {
        __u32 handle;
        __u64 ptr;
    } target;
    __u64 cookie;
    __u32 code;

    __u32 flags;
    pid_t sender_pid;
    uid_t sender_euid;
    __u64 data_size;
    __u64 offsets_size;

    union {
        struct {
            __u64 buffer;
            __u64 offsets;
        } ptr;
        __u8 buf[8];
    } data;
};