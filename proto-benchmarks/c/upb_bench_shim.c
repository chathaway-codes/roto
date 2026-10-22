#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#include "upb/mem/arena.h"
#include "benchmarks.upb.h"
#include "benchmark_message1_proto2.upb.h"
#include "benchmark_message1_proto3.upb.h"
#include "perftest_data.upb.h"

/* =========================================================================
 * Arena Lifecycle Helpers
 * ========================================================================= */

upb_Arena* upb_bench_arena_new(void) {
    return upb_Arena_New();
}

upb_Arena* upb_bench_arena_init(void* mem, size_t n) {
    return upb_Arena_Init(mem, n, &upb_alloc_global);
}

void upb_bench_arena_free(upb_Arena* arena) {
    upb_Arena_Free(arena);
}

/* =========================================================================
 * GoogleMessage1 (Proto2)
 * ========================================================================= */

benchmarks_proto2_GoogleMessage1* upb_gm1_proto2_parse(
    const uint8_t* buf, size_t len, upb_Arena* arena
) {
    return benchmarks_proto2_GoogleMessage1_parse((const char*)buf, len, arena);
}

uint64_t upb_gm1_proto2_partial_read(const benchmarks_proto2_GoogleMessage1* msg) {
    uint64_t acc = 0;
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field2(msg);
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field3(msg);
    upb_StringView s9 = benchmarks_proto2_GoogleMessage1_field9(msg);
    acc = acc * 31 + (uint64_t)s9.size;
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field67(msg);
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field100(msg);
    return acc;
}

uint64_t upb_gm1_proto2_traverse_all(const benchmarks_proto2_GoogleMessage1* msg) {
    uint64_t acc = 0;
    upb_StringView s;
    s = benchmarks_proto2_GoogleMessage1_field1(msg); acc += s.size;
    s = benchmarks_proto2_GoogleMessage1_field9(msg); acc += s.size;
    s = benchmarks_proto2_GoogleMessage1_field18(msg); acc += s.size;
    acc += benchmarks_proto2_GoogleMessage1_field80(msg);
    acc += benchmarks_proto2_GoogleMessage1_field81(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field2(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field3(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field280(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field6(msg);
    acc += (uint64_t)benchmarks_proto2_GoogleMessage1_field22(msg);
    s = benchmarks_proto2_GoogleMessage1_field4(msg); acc += s.size;
    size_t f5_len = 0;
    const uint64_t* f5 = benchmarks_proto2_GoogleMessage1_field5(msg, &f5_len);
    for (size_t i = 0; i < f5_len; i++) {
        acc += f5[i];
    }
    acc += benchmarks_proto2_GoogleMessage1_field59(msg);
    s = benchmarks_proto2_GoogleMessage1_field7(msg); acc += s.size;
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field16(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field130(msg);
    acc += benchmarks_proto2_GoogleMessage1_field12(msg);
    acc += benchmarks_proto2_GoogleMessage1_field17(msg);
    acc += benchmarks_proto2_GoogleMessage1_field13(msg);
    acc += benchmarks_proto2_GoogleMessage1_field14(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field104(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field100(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field101(msg);
    s = benchmarks_proto2_GoogleMessage1_field102(msg); acc += s.size;
    s = benchmarks_proto2_GoogleMessage1_field103(msg); acc += s.size;
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field29(msg);
    acc += benchmarks_proto2_GoogleMessage1_field30(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field60(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field271(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field272(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field150(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field23(msg);
    acc += benchmarks_proto2_GoogleMessage1_field24(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field25(msg);
    const benchmarks_proto2_GoogleMessage1SubMessage* sub = benchmarks_proto2_GoogleMessage1_field15(msg);
    if (sub) {
        acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1SubMessage_field1(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1SubMessage_field2(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1SubMessage_field3(sub);
        s = benchmarks_proto2_GoogleMessage1SubMessage_field15(sub); acc += s.size;
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field12(sub);
        acc += (uint64_t)benchmarks_proto2_GoogleMessage1SubMessage_field13(sub);
        acc += (uint64_t)benchmarks_proto2_GoogleMessage1SubMessage_field14(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1SubMessage_field16(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1SubMessage_field19(sub);
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field20(sub);
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field28(sub);
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field21(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1SubMessage_field22(sub);
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field23(sub);
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field206(sub);
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field203(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1SubMessage_field204(sub);
        s = benchmarks_proto2_GoogleMessage1SubMessage_field205(sub); acc += s.size;
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field207(sub);
        acc += benchmarks_proto2_GoogleMessage1SubMessage_field300(sub);
    }
    acc += benchmarks_proto2_GoogleMessage1_field78(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field67(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field68(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field128(msg);
    s = benchmarks_proto2_GoogleMessage1_field129(msg); acc += s.size;
    acc += (uint64_t)(uint32_t)benchmarks_proto2_GoogleMessage1_field131(msg);
    return acc;
}

const uint8_t* upb_gm1_proto2_serialize(
    const benchmarks_proto2_GoogleMessage1* msg, upb_Arena* arena, size_t* out_len
) {
    return (const uint8_t*)benchmarks_proto2_GoogleMessage1_serialize(msg, arena, out_len);
}

/* Construct and serialize GoogleMessage1 (proto2) from scratch (matching roto builder benchmark) */
const uint8_t* upb_gm1_proto2_build_and_serialize(
    const benchmarks_proto2_GoogleMessage1* src, upb_Arena* arena, size_t* out_len
) {
    benchmarks_proto2_GoogleMessage1* dst = benchmarks_proto2_GoogleMessage1_new(arena);
    if (benchmarks_proto2_GoogleMessage1_has_field1(src))
        benchmarks_proto2_GoogleMessage1_set_field1(dst, benchmarks_proto2_GoogleMessage1_field1(src));
    if (benchmarks_proto2_GoogleMessage1_has_field9(src))
        benchmarks_proto2_GoogleMessage1_set_field9(dst, benchmarks_proto2_GoogleMessage1_field9(src));
    if (benchmarks_proto2_GoogleMessage1_has_field18(src))
        benchmarks_proto2_GoogleMessage1_set_field18(dst, benchmarks_proto2_GoogleMessage1_field18(src));
    if (benchmarks_proto2_GoogleMessage1_has_field80(src))
        benchmarks_proto2_GoogleMessage1_set_field80(dst, benchmarks_proto2_GoogleMessage1_field80(src));
    if (benchmarks_proto2_GoogleMessage1_has_field81(src))
        benchmarks_proto2_GoogleMessage1_set_field81(dst, benchmarks_proto2_GoogleMessage1_field81(src));
    if (benchmarks_proto2_GoogleMessage1_has_field2(src))
        benchmarks_proto2_GoogleMessage1_set_field2(dst, benchmarks_proto2_GoogleMessage1_field2(src));
    if (benchmarks_proto2_GoogleMessage1_has_field3(src))
        benchmarks_proto2_GoogleMessage1_set_field3(dst, benchmarks_proto2_GoogleMessage1_field3(src));
    if (benchmarks_proto2_GoogleMessage1_has_field280(src))
        benchmarks_proto2_GoogleMessage1_set_field280(dst, benchmarks_proto2_GoogleMessage1_field280(src));
    if (benchmarks_proto2_GoogleMessage1_has_field6(src))
        benchmarks_proto2_GoogleMessage1_set_field6(dst, benchmarks_proto2_GoogleMessage1_field6(src));
    if (benchmarks_proto2_GoogleMessage1_has_field22(src))
        benchmarks_proto2_GoogleMessage1_set_field22(dst, benchmarks_proto2_GoogleMessage1_field22(src));
    if (benchmarks_proto2_GoogleMessage1_has_field4(src))
        benchmarks_proto2_GoogleMessage1_set_field4(dst, benchmarks_proto2_GoogleMessage1_field4(src));
    size_t f5_len = 0;
    const uint64_t* f5 = benchmarks_proto2_GoogleMessage1_field5(src, &f5_len);
    for (size_t i = 0; i < f5_len; i++) {
        benchmarks_proto2_GoogleMessage1_add_field5(dst, f5[i], arena);
    }
    if (benchmarks_proto2_GoogleMessage1_has_field59(src))
        benchmarks_proto2_GoogleMessage1_set_field59(dst, benchmarks_proto2_GoogleMessage1_field59(src));
    if (benchmarks_proto2_GoogleMessage1_has_field7(src))
        benchmarks_proto2_GoogleMessage1_set_field7(dst, benchmarks_proto2_GoogleMessage1_field7(src));
    if (benchmarks_proto2_GoogleMessage1_has_field16(src))
        benchmarks_proto2_GoogleMessage1_set_field16(dst, benchmarks_proto2_GoogleMessage1_field16(src));
    if (benchmarks_proto2_GoogleMessage1_has_field130(src))
        benchmarks_proto2_GoogleMessage1_set_field130(dst, benchmarks_proto2_GoogleMessage1_field130(src));
    if (benchmarks_proto2_GoogleMessage1_has_field12(src))
        benchmarks_proto2_GoogleMessage1_set_field12(dst, benchmarks_proto2_GoogleMessage1_field12(src));
    if (benchmarks_proto2_GoogleMessage1_has_field17(src))
        benchmarks_proto2_GoogleMessage1_set_field17(dst, benchmarks_proto2_GoogleMessage1_field17(src));
    if (benchmarks_proto2_GoogleMessage1_has_field13(src))
        benchmarks_proto2_GoogleMessage1_set_field13(dst, benchmarks_proto2_GoogleMessage1_field13(src));
    if (benchmarks_proto2_GoogleMessage1_has_field14(src))
        benchmarks_proto2_GoogleMessage1_set_field14(dst, benchmarks_proto2_GoogleMessage1_field14(src));
    if (benchmarks_proto2_GoogleMessage1_has_field104(src))
        benchmarks_proto2_GoogleMessage1_set_field104(dst, benchmarks_proto2_GoogleMessage1_field104(src));
    if (benchmarks_proto2_GoogleMessage1_has_field100(src))
        benchmarks_proto2_GoogleMessage1_set_field100(dst, benchmarks_proto2_GoogleMessage1_field100(src));
    if (benchmarks_proto2_GoogleMessage1_has_field101(src))
        benchmarks_proto2_GoogleMessage1_set_field101(dst, benchmarks_proto2_GoogleMessage1_field101(src));
    if (benchmarks_proto2_GoogleMessage1_has_field102(src))
        benchmarks_proto2_GoogleMessage1_set_field102(dst, benchmarks_proto2_GoogleMessage1_field102(src));
    if (benchmarks_proto2_GoogleMessage1_has_field103(src))
        benchmarks_proto2_GoogleMessage1_set_field103(dst, benchmarks_proto2_GoogleMessage1_field103(src));
    if (benchmarks_proto2_GoogleMessage1_has_field29(src))
        benchmarks_proto2_GoogleMessage1_set_field29(dst, benchmarks_proto2_GoogleMessage1_field29(src));
    if (benchmarks_proto2_GoogleMessage1_has_field30(src))
        benchmarks_proto2_GoogleMessage1_set_field30(dst, benchmarks_proto2_GoogleMessage1_field30(src));
    if (benchmarks_proto2_GoogleMessage1_has_field60(src))
        benchmarks_proto2_GoogleMessage1_set_field60(dst, benchmarks_proto2_GoogleMessage1_field60(src));
    if (benchmarks_proto2_GoogleMessage1_has_field271(src))
        benchmarks_proto2_GoogleMessage1_set_field271(dst, benchmarks_proto2_GoogleMessage1_field271(src));
    if (benchmarks_proto2_GoogleMessage1_has_field272(src))
        benchmarks_proto2_GoogleMessage1_set_field272(dst, benchmarks_proto2_GoogleMessage1_field272(src));
    if (benchmarks_proto2_GoogleMessage1_has_field150(src))
        benchmarks_proto2_GoogleMessage1_set_field150(dst, benchmarks_proto2_GoogleMessage1_field150(src));
    if (benchmarks_proto2_GoogleMessage1_has_field23(src))
        benchmarks_proto2_GoogleMessage1_set_field23(dst, benchmarks_proto2_GoogleMessage1_field23(src));
    if (benchmarks_proto2_GoogleMessage1_has_field24(src))
        benchmarks_proto2_GoogleMessage1_set_field24(dst, benchmarks_proto2_GoogleMessage1_field24(src));
    if (benchmarks_proto2_GoogleMessage1_has_field25(src))
        benchmarks_proto2_GoogleMessage1_set_field25(dst, benchmarks_proto2_GoogleMessage1_field25(src));
    if (benchmarks_proto2_GoogleMessage1_has_field15(src)) {
        const benchmarks_proto2_GoogleMessage1SubMessage* s_sub = benchmarks_proto2_GoogleMessage1_field15(src);
        benchmarks_proto2_GoogleMessage1SubMessage* d_sub = benchmarks_proto2_GoogleMessage1_mutable_field15(dst, arena);
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field1(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field1(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field1(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field2(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field2(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field2(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field3(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field3(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field3(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field15(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field15(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field15(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field12(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field12(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field12(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field13(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field13(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field13(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field14(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field14(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field14(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field16(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field16(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field16(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field19(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field19(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field19(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field20(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field20(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field20(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field28(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field28(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field28(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field21(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field21(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field21(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field22(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field22(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field22(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field23(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field23(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field23(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field206(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field206(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field206(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field203(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field203(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field203(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field204(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field204(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field204(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field205(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field205(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field205(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field207(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field207(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field207(s_sub));
        if (benchmarks_proto2_GoogleMessage1SubMessage_has_field300(s_sub))
            benchmarks_proto2_GoogleMessage1SubMessage_set_field300(d_sub, benchmarks_proto2_GoogleMessage1SubMessage_field300(s_sub));
    }
    if (benchmarks_proto2_GoogleMessage1_has_field78(src))
        benchmarks_proto2_GoogleMessage1_set_field78(dst, benchmarks_proto2_GoogleMessage1_field78(src));
    if (benchmarks_proto2_GoogleMessage1_has_field67(src))
        benchmarks_proto2_GoogleMessage1_set_field67(dst, benchmarks_proto2_GoogleMessage1_field67(src));
    if (benchmarks_proto2_GoogleMessage1_has_field68(src))
        benchmarks_proto2_GoogleMessage1_set_field68(dst, benchmarks_proto2_GoogleMessage1_field68(src));
    if (benchmarks_proto2_GoogleMessage1_has_field128(src))
        benchmarks_proto2_GoogleMessage1_set_field128(dst, benchmarks_proto2_GoogleMessage1_field128(src));
    if (benchmarks_proto2_GoogleMessage1_has_field129(src))
        benchmarks_proto2_GoogleMessage1_set_field129(dst, benchmarks_proto2_GoogleMessage1_field129(src));
    if (benchmarks_proto2_GoogleMessage1_has_field131(src))
        benchmarks_proto2_GoogleMessage1_set_field131(dst, benchmarks_proto2_GoogleMessage1_field131(src));

    return (const uint8_t*)benchmarks_proto2_GoogleMessage1_serialize(dst, arena, out_len);
}

/* =========================================================================
 * GoogleMessage1 (Proto3)
 * ========================================================================= */

benchmarks_proto3_GoogleMessage1* upb_gm1_proto3_parse(
    const uint8_t* buf, size_t len, upb_Arena* arena
) {
    return benchmarks_proto3_GoogleMessage1_parse((const char*)buf, len, arena);
}

uint64_t upb_gm1_proto3_partial_read(const benchmarks_proto3_GoogleMessage1* msg) {
    uint64_t acc = 0;
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field2(msg);
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field3(msg);
    upb_StringView s9 = benchmarks_proto3_GoogleMessage1_field9(msg);
    acc = acc * 31 + (uint64_t)s9.size;
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field67(msg);
    acc = acc * 31 + (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field100(msg);
    return acc;
}

uint64_t upb_gm1_proto3_traverse_all(const benchmarks_proto3_GoogleMessage1* msg) {
    uint64_t acc = 0;
    upb_StringView s;
    s = benchmarks_proto3_GoogleMessage1_field1(msg); acc += s.size;
    s = benchmarks_proto3_GoogleMessage1_field9(msg); acc += s.size;
    s = benchmarks_proto3_GoogleMessage1_field18(msg); acc += s.size;
    acc += benchmarks_proto3_GoogleMessage1_field80(msg);
    acc += benchmarks_proto3_GoogleMessage1_field81(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field2(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field3(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field280(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field6(msg);
    acc += (uint64_t)benchmarks_proto3_GoogleMessage1_field22(msg);
    s = benchmarks_proto3_GoogleMessage1_field4(msg); acc += s.size;
    size_t f5_len = 0;
    const uint64_t* f5 = benchmarks_proto3_GoogleMessage1_field5(msg, &f5_len);
    for (size_t i = 0; i < f5_len; i++) {
        acc += f5[i];
    }
    acc += benchmarks_proto3_GoogleMessage1_field59(msg);
    s = benchmarks_proto3_GoogleMessage1_field7(msg); acc += s.size;
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field16(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field130(msg);
    acc += benchmarks_proto3_GoogleMessage1_field12(msg);
    acc += benchmarks_proto3_GoogleMessage1_field17(msg);
    acc += benchmarks_proto3_GoogleMessage1_field13(msg);
    acc += benchmarks_proto3_GoogleMessage1_field14(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field104(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field100(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field101(msg);
    s = benchmarks_proto3_GoogleMessage1_field102(msg); acc += s.size;
    s = benchmarks_proto3_GoogleMessage1_field103(msg); acc += s.size;
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field29(msg);
    acc += benchmarks_proto3_GoogleMessage1_field30(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field60(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field271(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field272(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field150(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field23(msg);
    acc += benchmarks_proto3_GoogleMessage1_field24(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field25(msg);
    const benchmarks_proto3_GoogleMessage1SubMessage* sub = benchmarks_proto3_GoogleMessage1_field15(msg);
    if (sub) {
        acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1SubMessage_field1(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1SubMessage_field2(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1SubMessage_field3(sub);
        s = benchmarks_proto3_GoogleMessage1SubMessage_field15(sub); acc += s.size;
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field12(sub);
        acc += (uint64_t)benchmarks_proto3_GoogleMessage1SubMessage_field13(sub);
        acc += (uint64_t)benchmarks_proto3_GoogleMessage1SubMessage_field14(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1SubMessage_field16(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1SubMessage_field19(sub);
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field20(sub);
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field28(sub);
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field21(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1SubMessage_field22(sub);
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field23(sub);
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field206(sub);
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field203(sub);
        acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1SubMessage_field204(sub);
        s = benchmarks_proto3_GoogleMessage1SubMessage_field205(sub); acc += s.size;
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field207(sub);
        acc += benchmarks_proto3_GoogleMessage1SubMessage_field300(sub);
    }
    acc += benchmarks_proto3_GoogleMessage1_field78(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field67(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field68(msg);
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field128(msg);
    s = benchmarks_proto3_GoogleMessage1_field129(msg); acc += s.size;
    acc += (uint64_t)(uint32_t)benchmarks_proto3_GoogleMessage1_field131(msg);
    return acc;
}

const uint8_t* upb_gm1_proto3_serialize(
    const benchmarks_proto3_GoogleMessage1* msg, upb_Arena* arena, size_t* out_len
) {
    return (const uint8_t*)benchmarks_proto3_GoogleMessage1_serialize(msg, arena, out_len);
}

const uint8_t* upb_gm1_proto3_build_and_serialize(
    const benchmarks_proto3_GoogleMessage1* src, upb_Arena* arena, size_t* out_len
) {
    benchmarks_proto3_GoogleMessage1* dst = benchmarks_proto3_GoogleMessage1_new(arena);
    upb_StringView s;
    s = benchmarks_proto3_GoogleMessage1_field1(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field1(dst, s);
    s = benchmarks_proto3_GoogleMessage1_field9(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field9(dst, s);
    s = benchmarks_proto3_GoogleMessage1_field18(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field18(dst, s);
    if (benchmarks_proto3_GoogleMessage1_field80(src)) benchmarks_proto3_GoogleMessage1_set_field80(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field81(src)) benchmarks_proto3_GoogleMessage1_set_field81(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field2(src)) benchmarks_proto3_GoogleMessage1_set_field2(dst, benchmarks_proto3_GoogleMessage1_field2(src));
    if (benchmarks_proto3_GoogleMessage1_field3(src)) benchmarks_proto3_GoogleMessage1_set_field3(dst, benchmarks_proto3_GoogleMessage1_field3(src));
    if (benchmarks_proto3_GoogleMessage1_field280(src)) benchmarks_proto3_GoogleMessage1_set_field280(dst, benchmarks_proto3_GoogleMessage1_field280(src));
    if (benchmarks_proto3_GoogleMessage1_field6(src)) benchmarks_proto3_GoogleMessage1_set_field6(dst, benchmarks_proto3_GoogleMessage1_field6(src));
    if (benchmarks_proto3_GoogleMessage1_field22(src)) benchmarks_proto3_GoogleMessage1_set_field22(dst, benchmarks_proto3_GoogleMessage1_field22(src));
    s = benchmarks_proto3_GoogleMessage1_field4(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field4(dst, s);
    size_t f5_len = 0;
    const uint64_t* f5 = benchmarks_proto3_GoogleMessage1_field5(src, &f5_len);
    for (size_t i = 0; i < f5_len; i++) {
        benchmarks_proto3_GoogleMessage1_add_field5(dst, f5[i], arena);
    }
    if (benchmarks_proto3_GoogleMessage1_field59(src)) benchmarks_proto3_GoogleMessage1_set_field59(dst, true);
    s = benchmarks_proto3_GoogleMessage1_field7(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field7(dst, s);
    if (benchmarks_proto3_GoogleMessage1_field16(src)) benchmarks_proto3_GoogleMessage1_set_field16(dst, benchmarks_proto3_GoogleMessage1_field16(src));
    if (benchmarks_proto3_GoogleMessage1_field130(src)) benchmarks_proto3_GoogleMessage1_set_field130(dst, benchmarks_proto3_GoogleMessage1_field130(src));
    if (benchmarks_proto3_GoogleMessage1_field12(src)) benchmarks_proto3_GoogleMessage1_set_field12(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field17(src)) benchmarks_proto3_GoogleMessage1_set_field17(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field13(src)) benchmarks_proto3_GoogleMessage1_set_field13(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field14(src)) benchmarks_proto3_GoogleMessage1_set_field14(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field104(src)) benchmarks_proto3_GoogleMessage1_set_field104(dst, benchmarks_proto3_GoogleMessage1_field104(src));
    if (benchmarks_proto3_GoogleMessage1_field100(src)) benchmarks_proto3_GoogleMessage1_set_field100(dst, benchmarks_proto3_GoogleMessage1_field100(src));
    if (benchmarks_proto3_GoogleMessage1_field101(src)) benchmarks_proto3_GoogleMessage1_set_field101(dst, benchmarks_proto3_GoogleMessage1_field101(src));
    s = benchmarks_proto3_GoogleMessage1_field102(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field102(dst, s);
    s = benchmarks_proto3_GoogleMessage1_field103(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field103(dst, s);
    if (benchmarks_proto3_GoogleMessage1_field29(src)) benchmarks_proto3_GoogleMessage1_set_field29(dst, benchmarks_proto3_GoogleMessage1_field29(src));
    if (benchmarks_proto3_GoogleMessage1_field30(src)) benchmarks_proto3_GoogleMessage1_set_field30(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field60(src)) benchmarks_proto3_GoogleMessage1_set_field60(dst, benchmarks_proto3_GoogleMessage1_field60(src));
    if (benchmarks_proto3_GoogleMessage1_field271(src)) benchmarks_proto3_GoogleMessage1_set_field271(dst, benchmarks_proto3_GoogleMessage1_field271(src));
    if (benchmarks_proto3_GoogleMessage1_field272(src)) benchmarks_proto3_GoogleMessage1_set_field272(dst, benchmarks_proto3_GoogleMessage1_field272(src));
    if (benchmarks_proto3_GoogleMessage1_field150(src)) benchmarks_proto3_GoogleMessage1_set_field150(dst, benchmarks_proto3_GoogleMessage1_field150(src));
    if (benchmarks_proto3_GoogleMessage1_field23(src)) benchmarks_proto3_GoogleMessage1_set_field23(dst, benchmarks_proto3_GoogleMessage1_field23(src));
    if (benchmarks_proto3_GoogleMessage1_field24(src)) benchmarks_proto3_GoogleMessage1_set_field24(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field25(src)) benchmarks_proto3_GoogleMessage1_set_field25(dst, benchmarks_proto3_GoogleMessage1_field25(src));
    if (benchmarks_proto3_GoogleMessage1_has_field15(src)) {
        const benchmarks_proto3_GoogleMessage1SubMessage* s_sub = benchmarks_proto3_GoogleMessage1_field15(src);
        benchmarks_proto3_GoogleMessage1SubMessage* d_sub = benchmarks_proto3_GoogleMessage1_mutable_field15(dst, arena);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field1(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field1(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field1(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field2(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field2(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field2(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field3(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field3(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field3(s_sub));
        s = benchmarks_proto3_GoogleMessage1SubMessage_field15(s_sub);
        if (s.size) benchmarks_proto3_GoogleMessage1SubMessage_set_field15(d_sub, s);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field12(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field12(d_sub, true);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field13(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field13(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field13(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field14(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field14(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field14(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field16(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field16(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field16(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field19(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field19(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field19(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field20(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field20(d_sub, true);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field28(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field28(d_sub, true);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field21(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field21(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field21(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field22(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field22(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field22(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field23(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field23(d_sub, true);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field206(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field206(d_sub, true);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field203(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field203(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field203(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field204(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field204(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field204(s_sub));
        s = benchmarks_proto3_GoogleMessage1SubMessage_field205(s_sub);
        if (s.size) benchmarks_proto3_GoogleMessage1SubMessage_set_field205(d_sub, s);
        if (benchmarks_proto3_GoogleMessage1SubMessage_field207(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field207(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field207(s_sub));
        if (benchmarks_proto3_GoogleMessage1SubMessage_field300(s_sub))
            benchmarks_proto3_GoogleMessage1SubMessage_set_field300(d_sub, benchmarks_proto3_GoogleMessage1SubMessage_field300(s_sub));
    }
    if (benchmarks_proto3_GoogleMessage1_field78(src)) benchmarks_proto3_GoogleMessage1_set_field78(dst, true);
    if (benchmarks_proto3_GoogleMessage1_field67(src)) benchmarks_proto3_GoogleMessage1_set_field67(dst, benchmarks_proto3_GoogleMessage1_field67(src));
    if (benchmarks_proto3_GoogleMessage1_field68(src)) benchmarks_proto3_GoogleMessage1_set_field68(dst, benchmarks_proto3_GoogleMessage1_field68(src));
    if (benchmarks_proto3_GoogleMessage1_field128(src)) benchmarks_proto3_GoogleMessage1_set_field128(dst, benchmarks_proto3_GoogleMessage1_field128(src));
    s = benchmarks_proto3_GoogleMessage1_field129(src); if (s.size) benchmarks_proto3_GoogleMessage1_set_field129(dst, s);
    if (benchmarks_proto3_GoogleMessage1_field131(src)) benchmarks_proto3_GoogleMessage1_set_field131(dst, benchmarks_proto3_GoogleMessage1_field131(src));

    return (const uint8_t*)benchmarks_proto3_GoogleMessage1_serialize(dst, arena, out_len);
}

/* =========================================================================
 * PerftestData & Sub-Messages
 * ========================================================================= */

static uint64_t traverse_repeated_messages(const perftest_data_TestRepeatedMessages* msg) {
    uint64_t acc = 1;
    size_t n = 0;
    const perftest_data_TestRepeatedMessages* const* arr;
    arr = perftest_data_TestRepeatedMessages_messages1(msg, &n);
    for (size_t i = 0; i < n; i++) acc += traverse_repeated_messages(arr[i]);
    arr = perftest_data_TestRepeatedMessages_messages2(msg, &n);
    for (size_t i = 0; i < n; i++) acc += traverse_repeated_messages(arr[i]);
    arr = perftest_data_TestRepeatedMessages_messages3(msg, &n);
    for (size_t i = 0; i < n; i++) acc += traverse_repeated_messages(arr[i]);
    return acc;
}

static uint64_t traverse_optional_messages(const perftest_data_TestOptionalMessages* msg) {
    uint64_t acc = 1;
    const perftest_data_TestOptionalMessages* sub;
    sub = perftest_data_TestOptionalMessages_message1(msg);
    if (sub) acc += traverse_optional_messages(sub);
    sub = perftest_data_TestOptionalMessages_message2(msg);
    if (sub) acc += traverse_optional_messages(sub);
    sub = perftest_data_TestOptionalMessages_message3(msg);
    if (sub) acc += traverse_optional_messages(sub);
    return acc;
}

perftest_data_PerftestData* upb_perftest_parse(
    const uint8_t* buf, size_t len, upb_Arena* arena
) {
    return perftest_data_PerftestData_parse((const char*)buf, len, arena);
}

uint64_t upb_perftest_partial_read(const perftest_data_PerftestData* msg) {
    uint64_t acc = 0;
    size_t n = 0;
    const perftest_data_Test1* const* t1 = perftest_data_PerftestData_test1(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += (uint64_t)(uint32_t)perftest_data_Test1_value(t1[i]);
    }
    const perftest_data_TestStrings* const* ts = perftest_data_PerftestData_test_strings(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += perftest_data_TestStrings_s1(ts[i]).size;
    }
    return acc;
}

uint64_t upb_perftest_traverse_all(const perftest_data_PerftestData* msg) {
    uint64_t acc = 0;
    size_t n = 0;

    const perftest_data_Test1* const* t1 = perftest_data_PerftestData_test1(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += (uint64_t)(uint32_t)perftest_data_Test1_value(t1[i]);
    }

    const perftest_data_TestRepeatedBool* const* trb = perftest_data_PerftestData_test_repeated_bool(msg, &n);
    for (size_t i = 0; i < n; i++) {
        size_t m = 0;
        const bool* vals = perftest_data_TestRepeatedBool_values(trb[i], &m);
        for (size_t j = 0; j < m; j++) acc += vals[j];
    }

    const perftest_data_TestRepeatedMessages* const* trm = perftest_data_PerftestData_test_repeated_messages(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += traverse_repeated_messages(trm[i]);
    }

    const perftest_data_TestOptionalMessages* const* tom = perftest_data_PerftestData_test_optional_messages(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += traverse_optional_messages(tom[i]);
    }

    const perftest_data_TestStrings* const* ts = perftest_data_PerftestData_test_strings(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += perftest_data_TestStrings_s1(ts[i]).size;
        acc += perftest_data_TestStrings_s2(ts[i]).size;
        acc += perftest_data_TestStrings_s3(ts[i]).size;
    }

    const perftest_data_TestRepeatedPackedInt32* const* trp = perftest_data_PerftestData_test_repeated_packed_int32(msg, &n);
    for (size_t i = 0; i < n; i++) {
        size_t m = 0;
        const int32_t* vals = perftest_data_TestRepeatedPackedInt32_values(trp[i], &m);
        for (size_t j = 0; j < m; j++) acc += (uint64_t)(uint32_t)vals[j];
    }

    const perftest_data_TestBytes* const* tsb = perftest_data_PerftestData_test_small_bytearrays(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += perftest_data_TestBytes_b1(tsb[i]).size;
    }

    const perftest_data_TestBytes* const* tlb = perftest_data_PerftestData_test_large_bytearrays(msg, &n);
    for (size_t i = 0; i < n; i++) {
        acc += perftest_data_TestBytes_b1(tlb[i]).size;
    }

    return acc;
}

const uint8_t* upb_perftest_serialize(
    const perftest_data_PerftestData* msg, upb_Arena* arena, size_t* out_len
) {
    return (const uint8_t*)perftest_data_PerftestData_serialize(msg, arena, out_len);
}

/* Individual Perftest category helpers */
uint64_t upb_test1_parse_and_read(const uint8_t* buf, size_t len, upb_Arena* arena) {
    perftest_data_Test1* m = perftest_data_Test1_parse((const char*)buf, len, arena);
    return m ? (uint64_t)(uint32_t)perftest_data_Test1_value(m) : 0;
}

const uint8_t* upb_test1_build_and_serialize(int32_t val, bool has_val, upb_Arena* arena, size_t* out_len) {
    perftest_data_Test1* m = perftest_data_Test1_new(arena);
    if (has_val) perftest_data_Test1_set_value(m, val);
    return (const uint8_t*)perftest_data_Test1_serialize(m, arena, out_len);
}

uint64_t upb_test_repeated_bool_parse_and_read(const uint8_t* buf, size_t len, upb_Arena* arena) {
    perftest_data_TestRepeatedBool* m = perftest_data_TestRepeatedBool_parse((const char*)buf, len, arena);
    if (!m) return 0;
    size_t n = 0;
    const bool* vals = perftest_data_TestRepeatedBool_values(m, &n);
    uint64_t acc = 0;
    for (size_t i = 0; i < n; i++) acc += vals[i];
    return acc;
}

const uint8_t* upb_test_repeated_bool_build_and_serialize(const bool* vals, size_t n, upb_Arena* arena, size_t* out_len) {
    perftest_data_TestRepeatedBool* m = perftest_data_TestRepeatedBool_new(arena);
    for (size_t i = 0; i < n; i++) {
        perftest_data_TestRepeatedBool_add_values(m, vals[i], arena);
    }
    return (const uint8_t*)perftest_data_TestRepeatedBool_serialize(m, arena, out_len);
}

uint64_t upb_test_packed_int32_parse_and_read(const uint8_t* buf, size_t len, upb_Arena* arena) {
    perftest_data_TestRepeatedPackedInt32* m = perftest_data_TestRepeatedPackedInt32_parse((const char*)buf, len, arena);
    if (!m) return 0;
    size_t n = 0;
    const int32_t* vals = perftest_data_TestRepeatedPackedInt32_values(m, &n);
    uint64_t acc = 0;
    for (size_t i = 0; i < n; i++) acc += (uint64_t)(uint32_t)vals[i];
    return acc;
}

const uint8_t* upb_test_packed_int32_build_and_serialize(const int32_t* vals, size_t n, upb_Arena* arena, size_t* out_len) {
    perftest_data_TestRepeatedPackedInt32* m = perftest_data_TestRepeatedPackedInt32_new(arena);
    for (size_t i = 0; i < n; i++) {
        perftest_data_TestRepeatedPackedInt32_add_values(m, vals[i], arena);
    }
    return (const uint8_t*)perftest_data_TestRepeatedPackedInt32_serialize(m, arena, out_len);
}

uint64_t upb_test_strings_parse_and_read(const uint8_t* buf, size_t len, upb_Arena* arena) {
    perftest_data_TestStrings* m = perftest_data_TestStrings_parse((const char*)buf, len, arena);
    if (!m) return 0;
    return perftest_data_TestStrings_s1(m).size
         + perftest_data_TestStrings_s2(m).size
         + perftest_data_TestStrings_s3(m).size;
}

const uint8_t* upb_test_strings_build_and_serialize(
    const uint8_t* s1, size_t l1, bool has1,
    const uint8_t* s2, size_t l2, bool has2,
    const uint8_t* s3, size_t l3, bool has3,
    upb_Arena* arena, size_t* out_len
) {
    perftest_data_TestStrings* m = perftest_data_TestStrings_new(arena);
    if (has1) perftest_data_TestStrings_set_s1(m, upb_StringView_FromDataAndSize((const char*)s1, l1));
    if (has2) perftest_data_TestStrings_set_s2(m, upb_StringView_FromDataAndSize((const char*)s2, l2));
    if (has3) perftest_data_TestStrings_set_s3(m, upb_StringView_FromDataAndSize((const char*)s3, l3));
    return (const uint8_t*)perftest_data_TestStrings_serialize(m, arena, out_len);
}

uint64_t upb_test_bytes_parse_and_read(const uint8_t* buf, size_t len, upb_Arena* arena) {
    perftest_data_TestBytes* m = perftest_data_TestBytes_parse((const char*)buf, len, arena);
    if (!m) return 0;
    return perftest_data_TestBytes_b1(m).size;
}

const uint8_t* upb_test_bytes_build_and_serialize(
    const uint8_t* b1, size_t l1, bool has1,
    upb_Arena* arena, size_t* out_len
) {
    perftest_data_TestBytes* m = perftest_data_TestBytes_new(arena);
    if (has1) perftest_data_TestBytes_set_b1(m, upb_StringView_FromDataAndSize((const char*)b1, l1));
    return (const uint8_t*)perftest_data_TestBytes_serialize(m, arena, out_len);
}

uint64_t upb_test_optional_messages_parse_and_read(const uint8_t* buf, size_t len, upb_Arena* arena) {
    perftest_data_TestOptionalMessages* m = perftest_data_TestOptionalMessages_parse((const char*)buf, len, arena);
    return m ? traverse_optional_messages(m) : 0;
}

static void copy_optional_messages(
    const perftest_data_TestOptionalMessages* src,
    perftest_data_TestOptionalMessages* dst,
    upb_Arena* arena
) {
    if (perftest_data_TestOptionalMessages_has_message1(src)) {
        perftest_data_TestOptionalMessages* sub = perftest_data_TestOptionalMessages_mutable_message1(dst, arena);
        copy_optional_messages(perftest_data_TestOptionalMessages_message1(src), sub, arena);
    }
    if (perftest_data_TestOptionalMessages_has_message2(src)) {
        perftest_data_TestOptionalMessages* sub = perftest_data_TestOptionalMessages_mutable_message2(dst, arena);
        copy_optional_messages(perftest_data_TestOptionalMessages_message2(src), sub, arena);
    }
    if (perftest_data_TestOptionalMessages_has_message3(src)) {
        perftest_data_TestOptionalMessages* sub = perftest_data_TestOptionalMessages_mutable_message3(dst, arena);
        copy_optional_messages(perftest_data_TestOptionalMessages_message3(src), sub, arena);
    }
}

const uint8_t* upb_test_optional_messages_build_and_serialize(
    const perftest_data_TestOptionalMessages* src,
    upb_Arena* arena,
    size_t* out_len
) {
    perftest_data_TestOptionalMessages* dst = perftest_data_TestOptionalMessages_new(arena);
    copy_optional_messages(src, dst, arena);
    return (const uint8_t*)perftest_data_TestOptionalMessages_serialize(dst, arena, out_len);
}

perftest_data_TestOptionalMessages* upb_test_optional_messages_parse(
    const uint8_t* buf, size_t len, upb_Arena* arena
) {
    return perftest_data_TestOptionalMessages_parse((const char*)buf, len, arena);
}

uint64_t upb_test_repeated_messages_parse_and_read(const uint8_t* buf, size_t len, upb_Arena* arena) {
    perftest_data_TestRepeatedMessages* m = perftest_data_TestRepeatedMessages_parse((const char*)buf, len, arena);
    return m ? traverse_repeated_messages(m) : 0;
}

static void copy_repeated_messages(
    const perftest_data_TestRepeatedMessages* src,
    perftest_data_TestRepeatedMessages* dst,
    upb_Arena* arena
) {
    size_t n = 0;
    const perftest_data_TestRepeatedMessages* const* arr;
    arr = perftest_data_TestRepeatedMessages_messages1(src, &n);
    for (size_t i = 0; i < n; i++) {
        perftest_data_TestRepeatedMessages* sub = perftest_data_TestRepeatedMessages_add_messages1(dst, arena);
        copy_repeated_messages(arr[i], sub, arena);
    }
    arr = perftest_data_TestRepeatedMessages_messages2(src, &n);
    for (size_t i = 0; i < n; i++) {
        perftest_data_TestRepeatedMessages* sub = perftest_data_TestRepeatedMessages_add_messages2(dst, arena);
        copy_repeated_messages(arr[i], sub, arena);
    }
    arr = perftest_data_TestRepeatedMessages_messages3(src, &n);
    for (size_t i = 0; i < n; i++) {
        perftest_data_TestRepeatedMessages* sub = perftest_data_TestRepeatedMessages_add_messages3(dst, arena);
        copy_repeated_messages(arr[i], sub, arena);
    }
}

const uint8_t* upb_test_repeated_messages_build_and_serialize(
    const perftest_data_TestRepeatedMessages* src,
    upb_Arena* arena,
    size_t* out_len
) {
    perftest_data_TestRepeatedMessages* dst = perftest_data_TestRepeatedMessages_new(arena);
    copy_repeated_messages(src, dst, arena);
    return (const uint8_t*)perftest_data_TestRepeatedMessages_serialize(dst, arena, out_len);
}

perftest_data_TestRepeatedMessages* upb_test_repeated_messages_parse(
    const uint8_t* buf, size_t len, upb_Arena* arena
) {
    return perftest_data_TestRepeatedMessages_parse((const char*)buf, len, arena);
}
