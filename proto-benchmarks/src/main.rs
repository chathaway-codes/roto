use core::ffi::c_void;
use core::hint::black_box;
use std::time::{Duration, Instant};

use prost::Message as ProstMessage;
use proto_benchmarks::benchmarks;
use proto_benchmarks::datasets;
use proto_benchmarks::perftest_data;
use proto_benchmarks::prost_gen;
use proto_benchmarks::upb_ffi;

const MIN_BENCH_TIME: Duration = Duration::from_millis(250);
const WARMUP_TIME: Duration = Duration::from_millis(50);

struct BenchResult {
    ns_per_op: f64,
}

fn run_bench<F: FnMut()>(mut f: F) -> BenchResult {
    // Warmup
    let warmup_start = Instant::now();
    let mut warmup_iters = 0u64;
    while warmup_start.elapsed() < WARMUP_TIME {
        for _ in 0..64 {
            f();
        }
        warmup_iters += 64;
    }

    // Estimate batch size
    let batch = (warmup_iters / 10).clamp(1, 10_000);
    let mut total_iters = 0u64;
    let start = Instant::now();
    while start.elapsed() < MIN_BENCH_TIME {
        for _ in 0..batch {
            f();
        }
        total_iters += batch;
    }
    let elapsed = start.elapsed();
    BenchResult {
        ns_per_op: elapsed.as_nanos() as f64 / total_iters as f64,
    }
}

/* =========================================================================
 * GoogleMessage1 (Proto2) Helpers
 * ========================================================================= */

fn prost_gm1_proto2_partial(msg: &prost_gen::benchmarks::proto2::GoogleMessage1) -> u64 {
    let mut acc = 0u64;
    acc = acc.wrapping_mul(31).wrapping_add(msg.field2 as u32 as u64);
    acc = acc.wrapping_mul(31).wrapping_add(msg.field3 as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(msg.field9.as_deref().unwrap_or("").len() as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(msg.field67() as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(msg.field100() as u32 as u64);
    acc
}

fn roto_gm1_proto2_partial(view: &benchmarks::proto2::GoogleMessage1View<'_>) -> u64 {
    let mut acc = 0u64;
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field2().unwrap() as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field3().unwrap() as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field9().unwrap().len() as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field67().unwrap() as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field100().unwrap() as u32 as u64);
    acc
}

fn prost_gm1_proto2_traverse_all(msg: &prost_gen::benchmarks::proto2::GoogleMessage1) -> u64 {
    let mut acc = 0u64;
    acc = acc.wrapping_add(msg.field1.len() as u64);
    acc = acc.wrapping_add(msg.field9().len() as u64);
    acc = acc.wrapping_add(msg.field18().len() as u64);
    acc = acc.wrapping_add(msg.field80() as u64);
    acc = acc.wrapping_add(msg.field81() as u64);
    acc = acc.wrapping_add(msg.field2 as u32 as u64);
    acc = acc.wrapping_add(msg.field3 as u32 as u64);
    acc = acc.wrapping_add(msg.field280() as u32 as u64);
    acc = acc.wrapping_add(msg.field6() as u32 as u64);
    acc = acc.wrapping_add(msg.field22() as u64);
    acc = acc.wrapping_add(msg.field4().len() as u64);
    for &v in &msg.field5 {
        acc = acc.wrapping_add(v);
    }
    acc = acc.wrapping_add(msg.field59() as u64);
    acc = acc.wrapping_add(msg.field7().len() as u64);
    acc = acc.wrapping_add(msg.field16() as u32 as u64);
    acc = acc.wrapping_add(msg.field130() as u32 as u64);
    acc = acc.wrapping_add(msg.field12() as u64);
    acc = acc.wrapping_add(msg.field17() as u64);
    acc = acc.wrapping_add(msg.field13() as u64);
    acc = acc.wrapping_add(msg.field14() as u64);
    acc = acc.wrapping_add(msg.field104() as u32 as u64);
    acc = acc.wrapping_add(msg.field100() as u32 as u64);
    acc = acc.wrapping_add(msg.field101() as u32 as u64);
    acc = acc.wrapping_add(msg.field102().len() as u64);
    acc = acc.wrapping_add(msg.field103().len() as u64);
    acc = acc.wrapping_add(msg.field29() as u32 as u64);
    acc = acc.wrapping_add(msg.field30() as u64);
    acc = acc.wrapping_add(msg.field60() as u32 as u64);
    acc = acc.wrapping_add(msg.field271() as u32 as u64);
    acc = acc.wrapping_add(msg.field272() as u32 as u64);
    acc = acc.wrapping_add(msg.field150() as u32 as u64);
    acc = acc.wrapping_add(msg.field23() as u32 as u64);
    acc = acc.wrapping_add(msg.field24() as u64);
    acc = acc.wrapping_add(msg.field25() as u32 as u64);
    if let Some(sub) = &msg.field15 {
        acc = acc.wrapping_add(sub.field1() as u32 as u64);
        acc = acc.wrapping_add(sub.field2() as u32 as u64);
        acc = acc.wrapping_add(sub.field3() as u32 as u64);
        acc = acc.wrapping_add(sub.field15().len() as u64);
        acc = acc.wrapping_add(sub.field12() as u64);
        acc = acc.wrapping_add(sub.field13() as u64);
        acc = acc.wrapping_add(sub.field14() as u64);
        acc = acc.wrapping_add(sub.field16() as u32 as u64);
        acc = acc.wrapping_add(sub.field19() as u32 as u64);
        acc = acc.wrapping_add(sub.field20() as u64);
        acc = acc.wrapping_add(sub.field28() as u64);
        acc = acc.wrapping_add(sub.field21());
        acc = acc.wrapping_add(sub.field22() as u32 as u64);
        acc = acc.wrapping_add(sub.field23() as u64);
        acc = acc.wrapping_add(sub.field206() as u64);
        acc = acc.wrapping_add(sub.field203() as u64);
        acc = acc.wrapping_add(sub.field204() as u32 as u64);
        acc = acc.wrapping_add(sub.field205().len() as u64);
        acc = acc.wrapping_add(sub.field207());
        acc = acc.wrapping_add(sub.field300());
    }
    acc = acc.wrapping_add(msg.field78() as u64);
    acc = acc.wrapping_add(msg.field67() as u32 as u64);
    acc = acc.wrapping_add(msg.field68() as u32 as u64);
    acc = acc.wrapping_add(msg.field128() as u32 as u64);
    acc = acc.wrapping_add(msg.field129().len() as u64);
    acc = acc.wrapping_add(msg.field131() as u32 as u64);
    acc
}

fn roto_gm1_proto2_traverse_all(view: &benchmarks::proto2::GoogleMessage1View<'_>) -> u64 {
    let mut acc = 0u64;
    acc = acc.wrapping_add(view.field1().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field9().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field18().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field80().unwrap() as u64);
    acc = acc.wrapping_add(view.field81().unwrap() as u64);
    acc = acc.wrapping_add(view.field2().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field3().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field280().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field6().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field22().unwrap() as u64);
    acc = acc.wrapping_add(view.field4().unwrap().len() as u64);
    for item in view.field5() {
        acc = acc.wrapping_add(item.unwrap());
    }
    acc = acc.wrapping_add(view.field59().unwrap() as u64);
    acc = acc.wrapping_add(view.field7().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field16().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field130().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field12().unwrap() as u64);
    acc = acc.wrapping_add(view.field17().unwrap() as u64);
    acc = acc.wrapping_add(view.field13().unwrap() as u64);
    acc = acc.wrapping_add(view.field14().unwrap() as u64);
    acc = acc.wrapping_add(view.field104().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field100().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field101().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field102().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field103().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field29().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field30().unwrap() as u64);
    acc = acc.wrapping_add(view.field60().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field271().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field272().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field150().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field23().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field24().unwrap() as u64);
    acc = acc.wrapping_add(view.field25().unwrap() as u32 as u64);
    if let Some(sub) = view.field15().unwrap() {
        acc = acc.wrapping_add(sub.field1().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field2().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field3().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field15().unwrap().len() as u64);
        acc = acc.wrapping_add(sub.field12().unwrap() as u64);
        acc = acc.wrapping_add(sub.field13().unwrap() as u64);
        acc = acc.wrapping_add(sub.field14().unwrap() as u64);
        acc = acc.wrapping_add(sub.field16().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field19().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field20().unwrap() as u64);
        acc = acc.wrapping_add(sub.field28().unwrap() as u64);
        acc = acc.wrapping_add(sub.field21().unwrap());
        acc = acc.wrapping_add(sub.field22().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field23().unwrap() as u64);
        acc = acc.wrapping_add(sub.field206().unwrap() as u64);
        acc = acc.wrapping_add(sub.field203().unwrap() as u64);
        acc = acc.wrapping_add(sub.field204().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field205().unwrap().len() as u64);
        acc = acc.wrapping_add(sub.field207().unwrap());
        acc = acc.wrapping_add(sub.field300().unwrap());
    }
    acc = acc.wrapping_add(view.field78().unwrap() as u64);
    acc = acc.wrapping_add(view.field67().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field68().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field128().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field129().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field131().unwrap() as u32 as u64);
    acc
}

fn roto_gm1_proto2_encode<'b>(
    src: &prost_gen::benchmarks::proto2::GoogleMessage1,
    buf: &'b mut [u8],
) -> &'b [u8] {
    let mut b = benchmarks::proto2::GoogleMessage1Builder::new(buf);
    if let Some(v) = src.field131 {
        b.write_field131(v);
    }
    if let Some(v) = &src.field129 {
        b.write_field129(v);
    }
    if let Some(v) = src.field128 {
        b.write_field128(v);
    }
    if let Some(v) = src.field68 {
        b.write_field68(v);
    }
    if let Some(v) = src.field67 {
        b.write_field67(v);
    }
    if let Some(v) = src.field78 {
        b.write_field78(v);
    }
    if let Some(sub) = &src.field15 {
        b.write_field15(|sb| {
            if let Some(v) = sub.field300 {
                sb.write_field300(v);
            }
            if let Some(v) = sub.field207 {
                sb.write_field207(v);
            }
            if let Some(v) = &sub.field205 {
                sb.write_field205(v);
            }
            if let Some(v) = sub.field204 {
                sb.write_field204(v);
            }
            if let Some(v) = sub.field203 {
                sb.write_field203(v);
            }
            if let Some(v) = sub.field206 {
                sb.write_field206(v);
            }
            if let Some(v) = sub.field23 {
                sb.write_field23(v);
            }
            if let Some(v) = sub.field22 {
                sb.write_field22(v);
            }
            if let Some(v) = sub.field21 {
                sb.write_field21(v);
            }
            if let Some(v) = sub.field28 {
                sb.write_field28(v);
            }
            if let Some(v) = sub.field20 {
                sb.write_field20(v);
            }
            if let Some(v) = sub.field19 {
                sb.write_field19(v);
            }
            if let Some(v) = sub.field16 {
                sb.write_field16(v);
            }
            if let Some(v) = sub.field14 {
                sb.write_field14(v);
            }
            if let Some(v) = sub.field13 {
                sb.write_field13(v);
            }
            if let Some(v) = sub.field12 {
                sb.write_field12(v);
            }
            if let Some(v) = &sub.field15 {
                sb.write_field15(v);
            }
            if let Some(v) = sub.field3 {
                sb.write_field3(v);
            }
            if let Some(v) = sub.field2 {
                sb.write_field2(v);
            }
            if let Some(v) = sub.field1 {
                sb.write_field1(v);
            }
        });
    }
    if let Some(v) = src.field25 {
        b.write_field25(v);
    }
    if let Some(v) = src.field24 {
        b.write_field24(v);
    }
    if let Some(v) = src.field23 {
        b.write_field23(v);
    }
    if let Some(v) = src.field150 {
        b.write_field150(v);
    }
    if let Some(v) = src.field272 {
        b.write_field272(v);
    }
    if let Some(v) = src.field271 {
        b.write_field271(v);
    }
    if let Some(v) = src.field60 {
        b.write_field60(v);
    }
    if let Some(v) = src.field30 {
        b.write_field30(v);
    }
    if let Some(v) = src.field29 {
        b.write_field29(v);
    }
    if let Some(v) = &src.field103 {
        b.write_field103(v);
    }
    if let Some(v) = &src.field102 {
        b.write_field102(v);
    }
    if let Some(v) = src.field101 {
        b.write_field101(v);
    }
    if let Some(v) = src.field100 {
        b.write_field100(v);
    }
    if let Some(v) = src.field104 {
        b.write_field104(v);
    }
    if let Some(v) = src.field14 {
        b.write_field14(v);
    }
    if let Some(v) = src.field13 {
        b.write_field13(v);
    }
    if let Some(v) = src.field17 {
        b.write_field17(v);
    }
    if let Some(v) = src.field12 {
        b.write_field12(v);
    }
    if let Some(v) = src.field130 {
        b.write_field130(v);
    }
    if let Some(v) = src.field16 {
        b.write_field16(v);
    }
    if let Some(v) = &src.field7 {
        b.write_field7(v);
    }
    if let Some(v) = src.field59 {
        b.write_field59(v);
    }
    for &v in src.field5.iter().rev() {
        b.write_field5(v);
    }
    if let Some(v) = &src.field4 {
        b.write_field4(v);
    }
    if let Some(v) = src.field22 {
        b.write_field22(v);
    }
    if let Some(v) = src.field6 {
        b.write_field6(v);
    }
    if let Some(v) = src.field280 {
        b.write_field280(v);
    }
    b.write_field3(src.field3);
    b.write_field2(src.field2);
    if let Some(v) = src.field81 {
        b.write_field81(v);
    }
    if let Some(v) = src.field80 {
        b.write_field80(v);
    }
    if let Some(v) = &src.field18 {
        b.write_field18(v);
    }
    if let Some(v) = &src.field9 {
        b.write_field9(v);
    }
    b.write_field1(&src.field1);
    b.finish().unwrap()
}

/* =========================================================================
 * GoogleMessage1 (Proto3) Helpers
 * ========================================================================= */

fn prost_gm1_proto3_partial(msg: &prost_gen::benchmarks::proto3::GoogleMessage1) -> u64 {
    let mut acc = 0u64;
    acc = acc.wrapping_mul(31).wrapping_add(msg.field2 as u32 as u64);
    acc = acc.wrapping_mul(31).wrapping_add(msg.field3 as u32 as u64);
    acc = acc.wrapping_mul(31).wrapping_add(msg.field9.len() as u64);
    acc = acc.wrapping_mul(31).wrapping_add(msg.field67 as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(msg.field100 as u32 as u64);
    acc
}

fn roto_gm1_proto3_partial(view: &benchmarks::proto3::GoogleMessage1View<'_>) -> u64 {
    let mut acc = 0u64;
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field2().unwrap() as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field3().unwrap() as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field9().unwrap().len() as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field67().unwrap() as u32 as u64);
    acc = acc
        .wrapping_mul(31)
        .wrapping_add(view.field100().unwrap() as u32 as u64);
    acc
}

fn prost_gm1_proto3_traverse_all(msg: &prost_gen::benchmarks::proto3::GoogleMessage1) -> u64 {
    let mut acc = 0u64;
    acc = acc.wrapping_add(msg.field1.len() as u64);
    acc = acc.wrapping_add(msg.field9.len() as u64);
    acc = acc.wrapping_add(msg.field18.len() as u64);
    acc = acc.wrapping_add(msg.field80 as u64);
    acc = acc.wrapping_add(msg.field81 as u64);
    acc = acc.wrapping_add(msg.field2 as u32 as u64);
    acc = acc.wrapping_add(msg.field3 as u32 as u64);
    acc = acc.wrapping_add(msg.field280 as u32 as u64);
    acc = acc.wrapping_add(msg.field6 as u32 as u64);
    acc = acc.wrapping_add(msg.field22 as u64);
    acc = acc.wrapping_add(msg.field4.len() as u64);
    for &v in &msg.field5 {
        acc = acc.wrapping_add(v);
    }
    acc = acc.wrapping_add(msg.field59 as u64);
    acc = acc.wrapping_add(msg.field7.len() as u64);
    acc = acc.wrapping_add(msg.field16 as u32 as u64);
    acc = acc.wrapping_add(msg.field130 as u32 as u64);
    acc = acc.wrapping_add(msg.field12 as u64);
    acc = acc.wrapping_add(msg.field17 as u64);
    acc = acc.wrapping_add(msg.field13 as u64);
    acc = acc.wrapping_add(msg.field14 as u64);
    acc = acc.wrapping_add(msg.field104 as u32 as u64);
    acc = acc.wrapping_add(msg.field100 as u32 as u64);
    acc = acc.wrapping_add(msg.field101 as u32 as u64);
    acc = acc.wrapping_add(msg.field102.len() as u64);
    acc = acc.wrapping_add(msg.field103.len() as u64);
    acc = acc.wrapping_add(msg.field29 as u32 as u64);
    acc = acc.wrapping_add(msg.field30 as u64);
    acc = acc.wrapping_add(msg.field60 as u32 as u64);
    acc = acc.wrapping_add(msg.field271 as u32 as u64);
    acc = acc.wrapping_add(msg.field272 as u32 as u64);
    acc = acc.wrapping_add(msg.field150 as u32 as u64);
    acc = acc.wrapping_add(msg.field23 as u32 as u64);
    acc = acc.wrapping_add(msg.field24 as u64);
    acc = acc.wrapping_add(msg.field25 as u32 as u64);
    if let Some(sub) = &msg.field15 {
        acc = acc.wrapping_add(sub.field1 as u32 as u64);
        acc = acc.wrapping_add(sub.field2 as u32 as u64);
        acc = acc.wrapping_add(sub.field3 as u32 as u64);
        acc = acc.wrapping_add(sub.field15.len() as u64);
        acc = acc.wrapping_add(sub.field12 as u64);
        acc = acc.wrapping_add(sub.field13 as u64);
        acc = acc.wrapping_add(sub.field14 as u64);
        acc = acc.wrapping_add(sub.field16 as u32 as u64);
        acc = acc.wrapping_add(sub.field19 as u32 as u64);
        acc = acc.wrapping_add(sub.field20 as u64);
        acc = acc.wrapping_add(sub.field28 as u64);
        acc = acc.wrapping_add(sub.field21);
        acc = acc.wrapping_add(sub.field22 as u32 as u64);
        acc = acc.wrapping_add(sub.field23 as u64);
        acc = acc.wrapping_add(sub.field206 as u64);
        acc = acc.wrapping_add(sub.field203 as u64);
        acc = acc.wrapping_add(sub.field204 as u32 as u64);
        acc = acc.wrapping_add(sub.field205.len() as u64);
        acc = acc.wrapping_add(sub.field207);
        acc = acc.wrapping_add(sub.field300);
    }
    acc = acc.wrapping_add(msg.field78 as u64);
    acc = acc.wrapping_add(msg.field67 as u32 as u64);
    acc = acc.wrapping_add(msg.field68 as u32 as u64);
    acc = acc.wrapping_add(msg.field128 as u32 as u64);
    acc = acc.wrapping_add(msg.field129.len() as u64);
    acc = acc.wrapping_add(msg.field131 as u32 as u64);
    acc
}

fn roto_gm1_proto3_traverse_all(view: &benchmarks::proto3::GoogleMessage1View<'_>) -> u64 {
    let mut acc = 0u64;
    acc = acc.wrapping_add(view.field1().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field9().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field18().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field80().unwrap() as u64);
    acc = acc.wrapping_add(view.field81().unwrap() as u64);
    acc = acc.wrapping_add(view.field2().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field3().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field280().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field6().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field22().unwrap() as u64);
    acc = acc.wrapping_add(view.field4().unwrap().len() as u64);
    for item in view.field5() {
        acc = acc.wrapping_add(item.unwrap());
    }
    acc = acc.wrapping_add(view.field59().unwrap() as u64);
    acc = acc.wrapping_add(view.field7().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field16().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field130().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field12().unwrap() as u64);
    acc = acc.wrapping_add(view.field17().unwrap() as u64);
    acc = acc.wrapping_add(view.field13().unwrap() as u64);
    acc = acc.wrapping_add(view.field14().unwrap() as u64);
    acc = acc.wrapping_add(view.field104().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field100().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field101().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field102().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field103().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field29().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field30().unwrap() as u64);
    acc = acc.wrapping_add(view.field60().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field271().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field272().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field150().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field23().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field24().unwrap() as u64);
    acc = acc.wrapping_add(view.field25().unwrap() as u32 as u64);
    if let Some(sub) = view.field15().unwrap() {
        acc = acc.wrapping_add(sub.field1().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field2().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field3().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field15().unwrap().len() as u64);
        acc = acc.wrapping_add(sub.field12().unwrap() as u64);
        acc = acc.wrapping_add(sub.field13().unwrap() as u64);
        acc = acc.wrapping_add(sub.field14().unwrap() as u64);
        acc = acc.wrapping_add(sub.field16().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field19().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field20().unwrap() as u64);
        acc = acc.wrapping_add(sub.field28().unwrap() as u64);
        acc = acc.wrapping_add(sub.field21().unwrap());
        acc = acc.wrapping_add(sub.field22().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field23().unwrap() as u64);
        acc = acc.wrapping_add(sub.field206().unwrap() as u64);
        acc = acc.wrapping_add(sub.field203().unwrap() as u64);
        acc = acc.wrapping_add(sub.field204().unwrap() as u32 as u64);
        acc = acc.wrapping_add(sub.field205().unwrap().len() as u64);
        acc = acc.wrapping_add(sub.field207().unwrap());
        acc = acc.wrapping_add(sub.field300().unwrap());
    }
    acc = acc.wrapping_add(view.field78().unwrap() as u64);
    acc = acc.wrapping_add(view.field67().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field68().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field128().unwrap() as u32 as u64);
    acc = acc.wrapping_add(view.field129().unwrap().len() as u64);
    acc = acc.wrapping_add(view.field131().unwrap() as u32 as u64);
    acc
}

fn roto_gm1_proto3_encode<'b>(
    src: &prost_gen::benchmarks::proto3::GoogleMessage1,
    buf: &'b mut [u8],
) -> &'b [u8] {
    let mut b = benchmarks::proto3::GoogleMessage1Builder::new(buf);
    if src.field131 != 0 {
        b.write_field131(src.field131);
    }
    if !src.field129.is_empty() {
        b.write_field129(&src.field129);
    }
    if src.field128 != 0 {
        b.write_field128(src.field128);
    }
    if src.field68 != 0 {
        b.write_field68(src.field68);
    }
    if src.field67 != 0 {
        b.write_field67(src.field67);
    }
    if src.field78 {
        b.write_field78(true);
    }
    if let Some(sub) = &src.field15 {
        b.write_field15(|sb| {
            if sub.field300 != 0 {
                sb.write_field300(sub.field300);
            }
            if sub.field207 != 0 {
                sb.write_field207(sub.field207);
            }
            if !sub.field205.is_empty() {
                sb.write_field205(&sub.field205);
            }
            if sub.field204 != 0 {
                sb.write_field204(sub.field204);
            }
            if sub.field203 != 0 {
                sb.write_field203(sub.field203);
            }
            if sub.field206 {
                sb.write_field206(true);
            }
            if sub.field23 {
                sb.write_field23(true);
            }
            if sub.field22 != 0 {
                sb.write_field22(sub.field22);
            }
            if sub.field21 != 0 {
                sb.write_field21(sub.field21);
            }
            if sub.field28 {
                sb.write_field28(true);
            }
            if sub.field20 {
                sb.write_field20(true);
            }
            if sub.field19 != 0 {
                sb.write_field19(sub.field19);
            }
            if sub.field16 != 0 {
                sb.write_field16(sub.field16);
            }
            if sub.field14 != 0 {
                sb.write_field14(sub.field14);
            }
            if sub.field13 != 0 {
                sb.write_field13(sub.field13);
            }
            if sub.field12 {
                sb.write_field12(true);
            }
            if !sub.field15.is_empty() {
                sb.write_field15(&sub.field15);
            }
            if sub.field3 != 0 {
                sb.write_field3(sub.field3);
            }
            if sub.field2 != 0 {
                sb.write_field2(sub.field2);
            }
            if sub.field1 != 0 {
                sb.write_field1(sub.field1);
            }
        });
    }
    if src.field25 != 0 {
        b.write_field25(src.field25);
    }
    if src.field24 {
        b.write_field24(true);
    }
    if src.field23 != 0 {
        b.write_field23(src.field23);
    }
    if src.field150 != 0 {
        b.write_field150(src.field150);
    }
    if src.field272 != 0 {
        b.write_field272(src.field272);
    }
    if src.field271 != 0 {
        b.write_field271(src.field271);
    }
    if src.field60 != 0 {
        b.write_field60(src.field60);
    }
    if src.field30 {
        b.write_field30(true);
    }
    if src.field29 != 0 {
        b.write_field29(src.field29);
    }
    if !src.field103.is_empty() {
        b.write_field103(&src.field103);
    }
    if !src.field102.is_empty() {
        b.write_field102(&src.field102);
    }
    if src.field101 != 0 {
        b.write_field101(src.field101);
    }
    if src.field100 != 0 {
        b.write_field100(src.field100);
    }
    if src.field104 != 0 {
        b.write_field104(src.field104);
    }
    if src.field14 {
        b.write_field14(true);
    }
    if src.field13 {
        b.write_field13(true);
    }
    if src.field17 {
        b.write_field17(true);
    }
    if src.field12 {
        b.write_field12(true);
    }
    if src.field130 != 0 {
        b.write_field130(src.field130);
    }
    if src.field16 != 0 {
        b.write_field16(src.field16);
    }
    if !src.field7.is_empty() {
        b.write_field7(&src.field7);
    }
    if src.field59 {
        b.write_field59(true);
    }
    if !src.field5.is_empty() {
        b.write_packed_field5(&src.field5);
    }
    if !src.field4.is_empty() {
        b.write_field4(&src.field4);
    }
    if src.field22 != 0 {
        b.write_field22(src.field22);
    }
    if src.field6 != 0 {
        b.write_field6(src.field6);
    }
    if src.field280 != 0 {
        b.write_field280(src.field280);
    }
    if src.field3 != 0 {
        b.write_field3(src.field3);
    }
    if src.field2 != 0 {
        b.write_field2(src.field2);
    }
    if src.field81 {
        b.write_field81(true);
    }
    if src.field80 {
        b.write_field80(true);
    }
    if !src.field18.is_empty() {
        b.write_field18(&src.field18);
    }
    if !src.field9.is_empty() {
        b.write_field9(&src.field9);
    }
    if !src.field1.is_empty() {
        b.write_field1(&src.field1);
    }
    b.finish().unwrap()
}

/* =========================================================================
 * PerftestData Helpers
 * ========================================================================= */

fn prost_traverse_repeated_messages(
    msg: &prost_gen::perftest_data::TestRepeatedMessages,
) -> u64 {
    let mut acc = 1u64;
    for m in &msg.messages1 {
        acc += prost_traverse_repeated_messages(m);
    }
    for m in &msg.messages2 {
        acc += prost_traverse_repeated_messages(m);
    }
    for m in &msg.messages3 {
        acc += prost_traverse_repeated_messages(m);
    }
    acc
}

fn roto_traverse_repeated_messages(view: &perftest_data::TestRepeatedMessagesView<'_>) -> u64 {
    let mut acc = 1u64;
    for m in view.messages1() {
        acc += roto_traverse_repeated_messages(&m.unwrap());
    }
    for m in view.messages2() {
        acc += roto_traverse_repeated_messages(&m.unwrap());
    }
    for m in view.messages3() {
        acc += roto_traverse_repeated_messages(&m.unwrap());
    }
    acc
}

fn roto_encode_repeated_messages(
    src: &prost_gen::perftest_data::TestRepeatedMessages,
    b: &mut perftest_data::TestRepeatedMessagesBuilder<'_>,
) {
    for m in src.messages3.iter().rev() {
        b.write_messages3(|sb| roto_encode_repeated_messages(m, sb));
    }
    for m in src.messages2.iter().rev() {
        b.write_messages2(|sb| roto_encode_repeated_messages(m, sb));
    }
    for m in src.messages1.iter().rev() {
        b.write_messages1(|sb| roto_encode_repeated_messages(m, sb));
    }
}

fn prost_traverse_optional_messages(
    msg: &prost_gen::perftest_data::TestOptionalMessages,
) -> u64 {
    let mut acc = 1u64;
    if let Some(m) = &msg.message1 {
        acc += prost_traverse_optional_messages(m);
    }
    if let Some(m) = &msg.message2 {
        acc += prost_traverse_optional_messages(m);
    }
    if let Some(m) = &msg.message3 {
        acc += prost_traverse_optional_messages(m);
    }
    acc
}

fn roto_traverse_optional_messages(view: &perftest_data::TestOptionalMessagesView<'_>) -> u64 {
    let mut acc = 1u64;
    if let Some(m) = view.message1().unwrap() {
        acc += roto_traverse_optional_messages(&m);
    }
    if let Some(m) = view.message2().unwrap() {
        acc += roto_traverse_optional_messages(&m);
    }
    if let Some(m) = view.message3().unwrap() {
        acc += roto_traverse_optional_messages(&m);
    }
    acc
}

fn roto_encode_optional_messages(
    src: &prost_gen::perftest_data::TestOptionalMessages,
    b: &mut perftest_data::TestOptionalMessagesBuilder<'_>,
) {
    if let Some(m) = &src.message3 {
        b.write_message3(|sb| roto_encode_optional_messages(m, sb));
    }
    if let Some(m) = &src.message2 {
        b.write_message2(|sb| roto_encode_optional_messages(m, sb));
    }
    if let Some(m) = &src.message1 {
        b.write_message1(|sb| roto_encode_optional_messages(m, sb));
    }
}

fn prost_perftest_partial(msg: &prost_gen::perftest_data::PerftestData) -> u64 {
    let mut acc = 0u64;
    for m in &msg.test1 {
        acc = acc.wrapping_add(m.value() as u32 as u64);
    }
    for m in &msg.test_strings {
        acc = acc.wrapping_add(m.s1().len() as u64);
    }
    acc
}

fn roto_perftest_partial(view: &perftest_data::PerftestDataView<'_>) -> u64 {
    let mut acc = 0u64;
    for m in view.test1() {
        let m = m.unwrap();
        acc = acc.wrapping_add(m.value().unwrap() as u32 as u64);
    }
    for m in view.test_strings() {
        let m = m.unwrap();
        acc = acc.wrapping_add(m.s1().unwrap().len() as u64);
    }
    acc
}

fn prost_perftest_traverse_all(msg: &prost_gen::perftest_data::PerftestData) -> u64 {
    let mut acc = 0u64;
    for m in &msg.test1 {
        acc = acc.wrapping_add(m.value() as u32 as u64);
    }
    for m in &msg.test_repeated_bool {
        for &v in &m.values {
            acc = acc.wrapping_add(v as u64);
        }
    }
    for m in &msg.test_repeated_messages {
        acc = acc.wrapping_add(prost_traverse_repeated_messages(m));
    }
    for m in &msg.test_optional_messages {
        acc = acc.wrapping_add(prost_traverse_optional_messages(m));
    }
    for m in &msg.test_strings {
        acc = acc.wrapping_add(m.s1().len() as u64);
        acc = acc.wrapping_add(m.s2().len() as u64);
        acc = acc.wrapping_add(m.s3().len() as u64);
    }
    for m in &msg.test_repeated_packed_int32 {
        for &v in &m.values {
            acc = acc.wrapping_add(v as u32 as u64);
        }
    }
    for m in &msg.test_small_bytearrays {
        acc = acc.wrapping_add(m.b1().len() as u64);
    }
    for m in &msg.test_large_bytearrays {
        acc = acc.wrapping_add(m.b1().len() as u64);
    }
    acc
}

fn roto_perftest_traverse_all(view: &perftest_data::PerftestDataView<'_>) -> u64 {
    let mut acc = 0u64;
    for m in view.test1() {
        let m = m.unwrap();
        acc = acc.wrapping_add(m.value().unwrap() as u32 as u64);
    }
    for m in view.test_repeated_bool() {
        let m = m.unwrap();
        for v in m.values() {
            acc = acc.wrapping_add(v.unwrap() as u64);
        }
    }
    for m in view.test_repeated_messages() {
        let m = m.unwrap();
        acc = acc.wrapping_add(roto_traverse_repeated_messages(&m));
    }
    for m in view.test_optional_messages() {
        let m = m.unwrap();
        acc = acc.wrapping_add(roto_traverse_optional_messages(&m));
    }
    for m in view.test_strings() {
        let m = m.unwrap();
        acc = acc.wrapping_add(m.s1().unwrap().len() as u64);
        acc = acc.wrapping_add(m.s2().unwrap().len() as u64);
        acc = acc.wrapping_add(m.s3().unwrap().len() as u64);
    }
    for m in view.test_repeated_packed_int32() {
        let m = m.unwrap();
        for v in m.values() {
            acc = acc.wrapping_add(v.unwrap() as u32 as u64);
        }
    }
    for m in view.test_small_bytearrays() {
        let m = m.unwrap();
        acc = acc.wrapping_add(m.b1().unwrap().len() as u64);
    }
    for m in view.test_large_bytearrays() {
        let m = m.unwrap();
        acc = acc.wrapping_add(m.b1().unwrap().len() as u64);
    }
    acc
}

fn roto_perftest_encode<'b>(
    src: &prost_gen::perftest_data::PerftestData,
    buf: &'b mut [u8],
) -> &'b [u8] {
    let mut b = perftest_data::PerftestDataBuilder::new(buf);
    for m in src.test_large_bytearrays.iter().rev() {
        b.write_test_large_bytearrays(|sb| {
            if let Some(v) = &m.b1 {
                sb.write_b1(v);
            }
        });
    }
    for m in src.test_small_bytearrays.iter().rev() {
        b.write_test_small_bytearrays(|sb| {
            if let Some(v) = &m.b1 {
                sb.write_b1(v);
            }
        });
    }
    for m in src.test_repeated_packed_int32.iter().rev() {
        b.write_test_repeated_packed_int32(|sb| {
            if !m.values.is_empty() {
                sb.write_packed_values(&m.values);
            }
        });
    }
    for m in src.test_strings.iter().rev() {
        b.write_test_strings(|sb| {
            if let Some(v) = &m.s3 {
                sb.write_s3(v);
            }
            if let Some(v) = &m.s2 {
                sb.write_s2(v);
            }
            if let Some(v) = &m.s1 {
                sb.write_s1(v);
            }
        });
    }
    for m in src.test_optional_messages.iter().rev() {
        b.write_test_optional_messages(|sb| roto_encode_optional_messages(m, sb));
    }
    for m in src.test_repeated_messages.iter().rev() {
        b.write_test_repeated_messages(|sb| roto_encode_repeated_messages(m, sb));
    }
    for m in src.test_repeated_bool.iter().rev() {
        b.write_test_repeated_bool(|sb| {
            for &v in m.values.iter().rev() {
                sb.write_values(v);
            }
        });
    }
    for m in src.test1.iter().rev() {
        b.write_test1(|sb| {
            if let Some(v) = m.value {
                sb.write_value(v);
            }
        });
    }
    b.finish().unwrap()
}

fn print_row(
    workload: &str,
    operation: &str,
    roto_ns: f64,
    upb_stack_ns: f64,
    upb_heap_ns: f64,
    prost_ns: f64,
) {
    println!(
        "| {:<28} | {:<20} | {:>10.1} ns | {:>10.1} ns ({:>5.2}x) | {:>10.1} ns ({:>5.2}x) | {:>10.1} ns ({:>5.2}x) |",
        workload,
        operation,
        roto_ns,
        upb_stack_ns,
        upb_stack_ns / roto_ns,
        upb_heap_ns,
        upb_heap_ns / roto_ns,
        prost_ns,
        prost_ns / roto_ns,
    );
}

fn main() {
    // 1. Load datasets
    let ds_gm1_p2 = prost_gen::benchmarks::BenchmarkDataset::decode(datasets::GOOGLE_MESSAGE1_PROTO2)
        .expect("decode gm1_proto2 dataset");
    let gm1_p2_payload = ds_gm1_p2.payload[0].as_slice();
    let gm1_p2_prost =
        prost_gen::benchmarks::proto2::GoogleMessage1::decode(gm1_p2_payload).unwrap();

    let ds_gm1_p3 = prost_gen::benchmarks::BenchmarkDataset::decode(datasets::GOOGLE_MESSAGE1_PROTO3)
        .expect("decode gm1_proto3 dataset");
    let gm1_p3_payload = ds_gm1_p3.payload[0].as_slice();
    let gm1_p3_prost =
        prost_gen::benchmarks::proto3::GoogleMessage1::decode(gm1_p3_payload).unwrap();

    let perftest_payload = datasets::PERFTEST_DATA;
    let perftest_prost =
        prost_gen::perftest_data::PerftestData::decode(perftest_payload).unwrap();

    // Verify Equivalence across roto, upb, and prost!
    unsafe {
        let arena = upb_ffi::upb_bench_arena_new();

        // GM1 Proto2 verification
        let gm1_p2_roto = benchmarks::proto2::GoogleMessage1View::from_bytes(gm1_p2_payload).unwrap();
        let gm1_p2_upb =
            upb_ffi::upb_gm1_proto2_parse(gm1_p2_payload.as_ptr(), gm1_p2_payload.len(), arena);
        assert!(!gm1_p2_upb.is_null());
        assert_eq!(
            roto_gm1_proto2_partial(&gm1_p2_roto),
            prost_gm1_proto2_partial(&gm1_p2_prost),
            "gm1_proto2 partial mismatch roto vs prost"
        );
        assert_eq!(
            roto_gm1_proto2_partial(&gm1_p2_roto),
            upb_ffi::upb_gm1_proto2_partial_read(gm1_p2_upb),
            "gm1_proto2 partial mismatch roto vs upb"
        );
        assert_eq!(
            roto_gm1_proto2_traverse_all(&gm1_p2_roto),
            prost_gm1_proto2_traverse_all(&gm1_p2_prost),
            "gm1_proto2 full traverse mismatch roto vs prost"
        );
        assert_eq!(
            roto_gm1_proto2_traverse_all(&gm1_p2_roto),
            upb_ffi::upb_gm1_proto2_traverse_all(gm1_p2_upb),
            "gm1_proto2 full traverse mismatch roto vs upb"
        );

        let mut buf = [0u8; 1024];
        let roto_encoded = roto_gm1_proto2_encode(&gm1_p2_prost, &mut buf);
        let roundtrip_p2 =
            prost_gen::benchmarks::proto2::GoogleMessage1::decode(roto_encoded).unwrap();
        assert_eq!(roundtrip_p2, gm1_p2_prost, "gm1_proto2 roundtrip mismatch");

        // GM1 Proto3 verification
        let gm1_p3_roto = benchmarks::proto3::GoogleMessage1View::from_bytes(gm1_p3_payload).unwrap();
        let gm1_p3_upb =
            upb_ffi::upb_gm1_proto3_parse(gm1_p3_payload.as_ptr(), gm1_p3_payload.len(), arena);
        assert!(!gm1_p3_upb.is_null());
        assert_eq!(
            roto_gm1_proto3_partial(&gm1_p3_roto),
            prost_gm1_proto3_partial(&gm1_p3_prost),
            "gm1_proto3 partial mismatch roto vs prost"
        );
        assert_eq!(
            roto_gm1_proto3_partial(&gm1_p3_roto),
            upb_ffi::upb_gm1_proto3_partial_read(gm1_p3_upb),
            "gm1_proto3 partial mismatch roto vs upb"
        );
        assert_eq!(
            roto_gm1_proto3_traverse_all(&gm1_p3_roto),
            prost_gm1_proto3_traverse_all(&gm1_p3_prost),
            "gm1_proto3 full traverse mismatch roto vs prost"
        );
        assert_eq!(
            roto_gm1_proto3_traverse_all(&gm1_p3_roto),
            upb_ffi::upb_gm1_proto3_traverse_all(gm1_p3_upb),
            "gm1_proto3 full traverse mismatch roto vs upb"
        );

        let roto_encoded_p3 = roto_gm1_proto3_encode(&gm1_p3_prost, &mut buf);
        let roundtrip_p3 =
            prost_gen::benchmarks::proto3::GoogleMessage1::decode(roto_encoded_p3).unwrap();
        assert_eq!(roundtrip_p3, gm1_p3_prost, "gm1_proto3 roundtrip mismatch");

        // PerftestData verification
        let pf_roto = perftest_data::PerftestDataView::from_bytes(perftest_payload).unwrap();
        let pf_upb =
            upb_ffi::upb_perftest_parse(perftest_payload.as_ptr(), perftest_payload.len(), arena);
        assert!(!pf_upb.is_null());
        assert_eq!(
            roto_perftest_partial(&pf_roto),
            prost_perftest_partial(&perftest_prost),
            "perftest partial mismatch roto vs prost"
        );
        assert_eq!(
            roto_perftest_partial(&pf_roto),
            upb_ffi::upb_perftest_partial_read(pf_upb),
            "perftest partial mismatch roto vs upb"
        );
        assert_eq!(
            roto_perftest_traverse_all(&pf_roto),
            prost_perftest_traverse_all(&perftest_prost),
            "perftest full traverse mismatch roto vs prost"
        );
        assert_eq!(
            roto_perftest_traverse_all(&pf_roto),
            upb_ffi::upb_perftest_traverse_all(pf_upb),
            "perftest full traverse mismatch roto vs upb"
        );

        let mut pf_buf = vec![0u8; 256 * 1024];
        let pf_encoded = roto_perftest_encode(&perftest_prost, &mut pf_buf);
        let roundtrip_pf = prost_gen::perftest_data::PerftestData::decode(pf_encoded).unwrap();
        assert_eq!(roundtrip_pf, perftest_prost, "perftest roundtrip mismatch");

        upb_ffi::upb_bench_arena_free(arena);
    }

    println!("All correctness & wire-roundtrip checks passed across roto, native C upb, and prost!\n");
    println!(
        "| {:<28} | {:<20} | {:>13} | {:>22} | {:>22} | {:>22} |",
        "Dataset / Workload",
        "Operation",
        "roto",
        "upb (stack arena)",
        "upb (heap arena)",
        "prost"
    );
    println!(
        "|{:-<30}|{:-<22}|{:-<15}|{:-<24}|{:-<24}|{:-<24}|",
        "", "", "", "", "", ""
    );

    // =========================================================================
    // 1. google_message1_proto2 (228 bytes)
    // =========================================================================
    {
        let mut arena_mem = [0u8; 4096];

        // A. Partial Read (parse + read 5 fields)
        let r_roto = run_bench(|| {
            let v = benchmarks::proto2::GoogleMessage1View::from_bytes(black_box(gm1_p2_payload)).unwrap();
            black_box(roto_gm1_proto2_partial(&v));
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let m = upb_ffi::upb_gm1_proto2_parse(
                black_box(gm1_p2_payload.as_ptr()),
                black_box(gm1_p2_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto2_partial_read(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let m = upb_ffi::upb_gm1_proto2_parse(
                black_box(gm1_p2_payload.as_ptr()),
                black_box(gm1_p2_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto2_partial_read(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let m = prost_gen::benchmarks::proto2::GoogleMessage1::decode(black_box(gm1_p2_payload)).unwrap();
            black_box(prost_gm1_proto2_partial(&m));
        });
        print_row(
            "google_message1_proto2",
            "Partial Read (5 flds)",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        // B. Full Deserialize (parse + traverse all fields)
        let r_roto = run_bench(|| {
            let v = benchmarks::proto2::GoogleMessage1View::from_bytes(black_box(gm1_p2_payload)).unwrap();
            black_box(roto_gm1_proto2_traverse_all(&v));
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let m = upb_ffi::upb_gm1_proto2_parse(
                black_box(gm1_p2_payload.as_ptr()),
                black_box(gm1_p2_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto2_traverse_all(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let m = upb_ffi::upb_gm1_proto2_parse(
                black_box(gm1_p2_payload.as_ptr()),
                black_box(gm1_p2_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto2_traverse_all(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let m = prost_gen::benchmarks::proto2::GoogleMessage1::decode(black_box(gm1_p2_payload)).unwrap();
            black_box(prost_gm1_proto2_traverse_all(&m));
        });
        print_row(
            "google_message1_proto2",
            "Full Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        // C. Full Serialize (into reused buffer / arena)
        let mut out_buf = [0u8; 1024];
        let mut prost_vec = Vec::with_capacity(1024);
        let persistent_arena = unsafe { upb_ffi::upb_bench_arena_new() };
        let upb_src_msg = unsafe {
            upb_ffi::upb_gm1_proto2_parse(
                gm1_p2_payload.as_ptr(),
                gm1_p2_payload.len(),
                persistent_arena,
            )
        };

        let r_roto = run_bench(|| {
            let bytes = roto_gm1_proto2_encode(black_box(&gm1_p2_prost), &mut out_buf);
            black_box(bytes);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto2_build_and_serialize(
                black_box(upb_src_msg),
                arena,
                &mut len,
            );
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto2_build_and_serialize(
                black_box(upb_src_msg),
                arena,
                &mut len,
            );
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            prost_vec.clear();
            black_box(&gm1_p2_prost).encode(&mut prost_vec).unwrap();
            black_box(prost_vec.as_slice());
        });
        print_row(
            "google_message1_proto2",
            "Serialize (Build+Enc)",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        // Also measure upb serialize-only (from pre-built upb_Message)
        let r_upb_ser_only = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto2_serialize(black_box(upb_src_msg), arena, &mut len);
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_ser_only_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto2_serialize(black_box(upb_src_msg), arena, &mut len);
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        print_row(
            "google_message1_proto2",
            "Serialize (Pre-built)",
            r_roto.ns_per_op,
            r_upb_ser_only.ns_per_op,
            r_upb_ser_only_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        unsafe {
            upb_ffi::upb_bench_arena_free(persistent_arena);
        }
    }

    // =========================================================================
    // 2. google_message1_proto3 (222 bytes)
    // =========================================================================
    {
        let mut arena_mem = [0u8; 4096];

        let r_roto = run_bench(|| {
            let v = benchmarks::proto3::GoogleMessage1View::from_bytes(black_box(gm1_p3_payload)).unwrap();
            black_box(roto_gm1_proto3_partial(&v));
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let m = upb_ffi::upb_gm1_proto3_parse(
                black_box(gm1_p3_payload.as_ptr()),
                black_box(gm1_p3_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto3_partial_read(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let m = upb_ffi::upb_gm1_proto3_parse(
                black_box(gm1_p3_payload.as_ptr()),
                black_box(gm1_p3_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto3_partial_read(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let m = prost_gen::benchmarks::proto3::GoogleMessage1::decode(black_box(gm1_p3_payload)).unwrap();
            black_box(prost_gm1_proto3_partial(&m));
        });
        print_row(
            "google_message1_proto3",
            "Partial Read (5 flds)",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let r_roto = run_bench(|| {
            let v = benchmarks::proto3::GoogleMessage1View::from_bytes(black_box(gm1_p3_payload)).unwrap();
            black_box(roto_gm1_proto3_traverse_all(&v));
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let m = upb_ffi::upb_gm1_proto3_parse(
                black_box(gm1_p3_payload.as_ptr()),
                black_box(gm1_p3_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto3_traverse_all(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let m = upb_ffi::upb_gm1_proto3_parse(
                black_box(gm1_p3_payload.as_ptr()),
                black_box(gm1_p3_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_gm1_proto3_traverse_all(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let m = prost_gen::benchmarks::proto3::GoogleMessage1::decode(black_box(gm1_p3_payload)).unwrap();
            black_box(prost_gm1_proto3_traverse_all(&m));
        });
        print_row(
            "google_message1_proto3",
            "Full Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut out_buf = [0u8; 1024];
        let mut prost_vec = Vec::with_capacity(1024);
        let persistent_arena = unsafe { upb_ffi::upb_bench_arena_new() };
        let upb_src_msg = unsafe {
            upb_ffi::upb_gm1_proto3_parse(
                gm1_p3_payload.as_ptr(),
                gm1_p3_payload.len(),
                persistent_arena,
            )
        };

        let r_roto = run_bench(|| {
            let bytes = roto_gm1_proto3_encode(black_box(&gm1_p3_prost), &mut out_buf);
            black_box(bytes);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto3_build_and_serialize(
                black_box(upb_src_msg),
                arena,
                &mut len,
            );
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto3_build_and_serialize(
                black_box(upb_src_msg),
                arena,
                &mut len,
            );
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            prost_vec.clear();
            black_box(&gm1_p3_prost).encode(&mut prost_vec).unwrap();
            black_box(prost_vec.as_slice());
        });
        print_row(
            "google_message1_proto3",
            "Serialize (Build+Enc)",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let r_upb_ser_only = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto3_serialize(black_box(upb_src_msg), arena, &mut len);
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_ser_only_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut len = 0usize;
            let ptr = upb_ffi::upb_gm1_proto3_serialize(black_box(upb_src_msg), arena, &mut len);
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        print_row(
            "google_message1_proto3",
            "Serialize (Pre-built)",
            r_roto.ns_per_op,
            r_upb_ser_only.ns_per_op,
            r_upb_ser_only_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        unsafe {
            upb_ffi::upb_bench_arena_free(persistent_arena);
        }
    }

    // =========================================================================
    // 3. perftest_data (Full 104 KB container + micro-categories)
    // =========================================================================
    {
        let mut arena_mem = vec![0u8; 256 * 1024];

        let r_roto = run_bench(|| {
            let v = perftest_data::PerftestDataView::from_bytes(black_box(perftest_payload)).unwrap();
            black_box(roto_perftest_partial(&v));
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let m = upb_ffi::upb_perftest_parse(
                black_box(perftest_payload.as_ptr()),
                black_box(perftest_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_perftest_partial_read(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let m = upb_ffi::upb_perftest_parse(
                black_box(perftest_payload.as_ptr()),
                black_box(perftest_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_perftest_partial_read(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let m = prost_gen::perftest_data::PerftestData::decode(black_box(perftest_payload)).unwrap();
            black_box(prost_perftest_partial(&m));
        });
        print_row(
            "perftest_data (104 KB)",
            "Partial Read",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let r_roto = run_bench(|| {
            let v = perftest_data::PerftestDataView::from_bytes(black_box(perftest_payload)).unwrap();
            black_box(roto_perftest_traverse_all(&v));
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let m = upb_ffi::upb_perftest_parse(
                black_box(perftest_payload.as_ptr()),
                black_box(perftest_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_perftest_traverse_all(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let m = upb_ffi::upb_perftest_parse(
                black_box(perftest_payload.as_ptr()),
                black_box(perftest_payload.len()),
                arena,
            );
            black_box(upb_ffi::upb_perftest_traverse_all(m));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let m = prost_gen::perftest_data::PerftestData::decode(black_box(perftest_payload)).unwrap();
            black_box(prost_perftest_traverse_all(&m));
        });
        print_row(
            "perftest_data (104 KB)",
            "Full Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut pf_out = vec![0u8; 256 * 1024];
        let mut prost_vec = Vec::with_capacity(256 * 1024);
        let persistent_arena = unsafe { upb_ffi::upb_bench_arena_new() };
        let upb_pf_src = unsafe {
            upb_ffi::upb_perftest_parse(
                perftest_payload.as_ptr(),
                perftest_payload.len(),
                persistent_arena,
            )
        };

        let r_roto = run_bench(|| {
            let bytes = roto_perftest_encode(black_box(&perftest_prost), &mut pf_out);
            black_box(bytes);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut len = 0usize;
            let ptr = upb_ffi::upb_perftest_serialize(black_box(upb_pf_src), arena, &mut len);
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut len = 0usize;
            let ptr = upb_ffi::upb_perftest_serialize(black_box(upb_pf_src), arena, &mut len);
            black_box((ptr, len));
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            prost_vec.clear();
            black_box(&perftest_prost).encode(&mut prost_vec).unwrap();
            black_box(prost_vec.as_slice());
        });
        print_row(
            "perftest_data (104 KB)",
            "Full Serialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        unsafe {
            upb_ffi::upb_bench_arena_free(persistent_arena);
        }
    }

    // =========================================================================
    // 4. Perftest Sub-Categories (isolated feature micro-benchmarks)
    // =========================================================================
    println!();
    println!(
        "| {:<28} | {:<20} | {:>13} | {:>22} | {:>22} | {:>22} |",
        "Perftest Sub-Category",
        "Operation",
        "roto",
        "upb (stack arena)",
        "upb (heap arena)",
        "prost"
    );
    println!(
        "|{:-<30}|{:-<22}|{:-<15}|{:-<24}|{:-<24}|{:-<24}|",
        "", "", "", "", "", ""
    );

    let mut arena_mem = vec![0u8; 256 * 1024];

    // 4.1 test1 (small varint messages)
    {
        let payloads: Vec<Vec<u8>> = perftest_prost
            .test1
            .iter()
            .map(|m| m.encode_to_vec())
            .collect();
        let r_roto = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let v = perftest_data::Test1View::from_bytes(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(v.value().unwrap() as u32 as u64);
            }
            black_box(acc);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test1_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test1_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let m = prost_gen::perftest_data::Test1::decode(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(m.value() as u32 as u64);
            }
            black_box(acc);
        });
        print_row(
            "perftest::test1 (varints)",
            "Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut buf = [0u8; 64];
        let mut pvec = Vec::with_capacity(64);
        let r_roto = run_bench(|| {
            for m in &perftest_prost.test1 {
                let mut b = perftest_data::Test1Builder::new(&mut buf);
                if let Some(v) = m.value {
                    b.write_value(v);
                }
                black_box(b.finish().unwrap());
            }
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            for m in &perftest_prost.test1 {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test1_build_and_serialize(
                    m.value(),
                    m.value.is_some(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            for m in &perftest_prost.test1 {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test1_build_and_serialize(
                    m.value(),
                    m.value.is_some(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            for m in &perftest_prost.test1 {
                pvec.clear();
                m.encode(&mut pvec).unwrap();
                black_box(pvec.as_slice());
            }
        });
        print_row(
            "perftest::test1 (varints)",
            "Serialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );
    }

    // 4.2 test_repeated_packed_int32
    {
        let payloads: Vec<Vec<u8>> = perftest_prost
            .test_repeated_packed_int32
            .iter()
            .map(|m| m.encode_to_vec())
            .collect();
        let r_roto = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let v = perftest_data::TestRepeatedPackedInt32View::from_bytes(black_box(p.as_slice())).unwrap();
                for item in v.values() {
                    acc = acc.wrapping_add(item.unwrap() as u32 as u64);
                }
            }
            black_box(acc);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_packed_int32_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_packed_int32_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let m = prost_gen::perftest_data::TestRepeatedPackedInt32::decode(black_box(p.as_slice())).unwrap();
                for &item in &m.values {
                    acc = acc.wrapping_add(item as u32 as u64);
                }
            }
            black_box(acc);
        });
        print_row(
            "perftest::packed_int32",
            "Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut buf = [0u8; 256];
        let mut pvec = Vec::with_capacity(256);
        let r_roto = run_bench(|| {
            for m in &perftest_prost.test_repeated_packed_int32 {
                let mut b = perftest_data::TestRepeatedPackedInt32Builder::new(&mut buf);
                if !m.values.is_empty() {
                    b.write_packed_values(&m.values);
                }
                black_box(b.finish().unwrap());
            }
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            for m in &perftest_prost.test_repeated_packed_int32 {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test_packed_int32_build_and_serialize(
                    m.values.as_ptr(),
                    m.values.len(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            for m in &perftest_prost.test_repeated_packed_int32 {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test_packed_int32_build_and_serialize(
                    m.values.as_ptr(),
                    m.values.len(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            for m in &perftest_prost.test_repeated_packed_int32 {
                pvec.clear();
                m.encode(&mut pvec).unwrap();
                black_box(pvec.as_slice());
            }
        });
        print_row(
            "perftest::packed_int32",
            "Serialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );
    }

    // 4.3 test_strings (zero-copy &str vs String)
    {
        let payloads: Vec<Vec<u8>> = perftest_prost
            .test_strings
            .iter()
            .map(|m| m.encode_to_vec())
            .collect();
        let r_roto = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let v = perftest_data::TestStringsView::from_bytes(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(v.s1().unwrap().len() as u64);
                acc = acc.wrapping_add(v.s2().unwrap().len() as u64);
                acc = acc.wrapping_add(v.s3().unwrap().len() as u64);
            }
            black_box(acc);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_strings_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_strings_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let m = prost_gen::perftest_data::TestStrings::decode(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(m.s1().len() as u64);
                acc = acc.wrapping_add(m.s2().len() as u64);
                acc = acc.wrapping_add(m.s3().len() as u64);
            }
            black_box(acc);
        });
        print_row(
            "perftest::test_strings",
            "Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut buf = [0u8; 256];
        let mut pvec = Vec::with_capacity(256);
        let r_roto = run_bench(|| {
            for m in &perftest_prost.test_strings {
                let mut b = perftest_data::TestStringsBuilder::new(&mut buf);
                if let Some(v) = &m.s3 {
                    b.write_s3(v);
                }
                if let Some(v) = &m.s2 {
                    b.write_s2(v);
                }
                if let Some(v) = &m.s1 {
                    b.write_s1(v);
                }
                black_box(b.finish().unwrap());
            }
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            for m in &perftest_prost.test_strings {
                let mut len = 0usize;
                let s1 = m.s1();
                let s2 = m.s2();
                let s3 = m.s3();
                let ptr = upb_ffi::upb_test_strings_build_and_serialize(
                    s1.as_ptr(),
                    s1.len(),
                    m.s1.is_some(),
                    s2.as_ptr(),
                    s2.len(),
                    m.s2.is_some(),
                    s3.as_ptr(),
                    s3.len(),
                    m.s3.is_some(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            for m in &perftest_prost.test_strings {
                let mut len = 0usize;
                let s1 = m.s1();
                let s2 = m.s2();
                let s3 = m.s3();
                let ptr = upb_ffi::upb_test_strings_build_and_serialize(
                    s1.as_ptr(),
                    s1.len(),
                    m.s1.is_some(),
                    s2.as_ptr(),
                    s2.len(),
                    m.s2.is_some(),
                    s3.as_ptr(),
                    s3.len(),
                    m.s3.is_some(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            for m in &perftest_prost.test_strings {
                pvec.clear();
                m.encode(&mut pvec).unwrap();
                black_box(pvec.as_slice());
            }
        });
        print_row(
            "perftest::test_strings",
            "Serialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );
    }

    // 4.4 test_optional_messages (deep recursive optional messages)
    {
        let payloads: Vec<Vec<u8>> = perftest_prost
            .test_optional_messages
            .iter()
            .map(|m| m.encode_to_vec())
            .collect();
        let r_roto = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let v = perftest_data::TestOptionalMessagesView::from_bytes(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(roto_traverse_optional_messages(&v));
            }
            black_box(acc);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_optional_messages_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_optional_messages_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let m = prost_gen::perftest_data::TestOptionalMessages::decode(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(prost_traverse_optional_messages(&m));
            }
            black_box(acc);
        });
        print_row(
            "perftest::optional_msgs",
            "Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut buf = [0u8; 256];
        let mut pvec = Vec::with_capacity(256);
        let persistent_arena = unsafe { upb_ffi::upb_bench_arena_new() };
        let upb_srcs: Vec<*mut upb_ffi::UpbTestOptionalMessages> = payloads
            .iter()
            .map(|p| unsafe {
                upb_ffi::upb_test_optional_messages_parse(p.as_ptr(), p.len(), persistent_arena)
            })
            .collect();

        let r_roto = run_bench(|| {
            for m in &perftest_prost.test_optional_messages {
                let mut b = perftest_data::TestOptionalMessagesBuilder::new(&mut buf);
                roto_encode_optional_messages(m, &mut b);
                black_box(b.finish().unwrap());
            }
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            for &src in &upb_srcs {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test_optional_messages_build_and_serialize(
                    black_box(src),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            for &src in &upb_srcs {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test_optional_messages_build_and_serialize(
                    black_box(src),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            for m in &perftest_prost.test_optional_messages {
                pvec.clear();
                m.encode(&mut pvec).unwrap();
                black_box(pvec.as_slice());
            }
        });
        print_row(
            "perftest::optional_msgs",
            "Serialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );
        unsafe {
            upb_ffi::upb_bench_arena_free(persistent_arena);
        }
    }

    // 4.5 test_repeated_messages
    {
        let payloads: Vec<Vec<u8>> = perftest_prost
            .test_repeated_messages
            .iter()
            .map(|m| m.encode_to_vec())
            .collect();
        let r_roto = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let v = perftest_data::TestRepeatedMessagesView::from_bytes(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(roto_traverse_repeated_messages(&v));
            }
            black_box(acc);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_repeated_messages_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_repeated_messages_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let m = prost_gen::perftest_data::TestRepeatedMessages::decode(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(prost_traverse_repeated_messages(&m));
            }
            black_box(acc);
        });
        print_row(
            "perftest::repeated_msgs",
            "Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut buf = [0u8; 256];
        let mut pvec = Vec::with_capacity(256);
        let persistent_arena = unsafe { upb_ffi::upb_bench_arena_new() };
        let upb_srcs: Vec<*mut upb_ffi::UpbTestRepeatedMessages> = payloads
            .iter()
            .map(|p| unsafe {
                upb_ffi::upb_test_repeated_messages_parse(p.as_ptr(), p.len(), persistent_arena)
            })
            .collect();

        let r_roto = run_bench(|| {
            for m in &perftest_prost.test_repeated_messages {
                let mut b = perftest_data::TestRepeatedMessagesBuilder::new(&mut buf);
                roto_encode_repeated_messages(m, &mut b);
                black_box(b.finish().unwrap());
            }
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            for &src in &upb_srcs {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test_repeated_messages_build_and_serialize(
                    black_box(src),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            for &src in &upb_srcs {
                let mut len = 0usize;
                let ptr = upb_ffi::upb_test_repeated_messages_build_and_serialize(
                    black_box(src),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            for m in &perftest_prost.test_repeated_messages {
                pvec.clear();
                m.encode(&mut pvec).unwrap();
                black_box(pvec.as_slice());
            }
        });
        print_row(
            "perftest::repeated_msgs",
            "Serialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );
        unsafe {
            upb_ffi::upb_bench_arena_free(persistent_arena);
        }
    }

    // 4.6 test_small_bytearrays (1 KB) & test_large_bytearrays (100 KB)
    for (label, list, buf_cap) in [
        (
            "perftest::small_bytes(1K)",
            &perftest_prost.test_small_bytearrays,
            2048usize,
        ),
        (
            "perftest::large_bytes(100K)",
            &perftest_prost.test_large_bytearrays,
            131072usize,
        ),
    ] {
        let payloads: Vec<Vec<u8>> = list.iter().map(|m| m.encode_to_vec()).collect();
        let r_roto = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let v = perftest_data::TestBytesView::from_bytes(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(v.b1().unwrap().len() as u64);
            }
            black_box(acc);
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_bytes_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            let mut acc = 0u64;
            for p in &payloads {
                acc = acc.wrapping_add(upb_ffi::upb_test_bytes_parse_and_read(
                    black_box(p.as_ptr()),
                    black_box(p.len()),
                    arena,
                ));
            }
            black_box(acc);
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            let mut acc = 0u64;
            for p in &payloads {
                let m = prost_gen::perftest_data::TestBytes::decode(black_box(p.as_slice())).unwrap();
                acc = acc.wrapping_add(m.b1().len() as u64);
            }
            black_box(acc);
        });
        print_row(
            label,
            "Deserialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );

        let mut buf = vec![0u8; buf_cap];
        let mut pvec = Vec::with_capacity(buf_cap);
        let r_roto = run_bench(|| {
            for m in list {
                let mut b = perftest_data::TestBytesBuilder::new(&mut buf);
                if let Some(v) = &m.b1 {
                    b.write_b1(v);
                }
                black_box(b.finish().unwrap());
            }
        });
        let r_upb_stack = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_init(
                arena_mem.as_mut_ptr() as *mut c_void,
                arena_mem.len(),
            );
            for m in list {
                let mut len = 0usize;
                let b1 = m.b1();
                let ptr = upb_ffi::upb_test_bytes_build_and_serialize(
                    b1.as_ptr(),
                    b1.len(),
                    m.b1.is_some(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_upb_heap = run_bench(|| unsafe {
            let arena = upb_ffi::upb_bench_arena_new();
            for m in list {
                let mut len = 0usize;
                let b1 = m.b1();
                let ptr = upb_ffi::upb_test_bytes_build_and_serialize(
                    b1.as_ptr(),
                    b1.len(),
                    m.b1.is_some(),
                    arena,
                    &mut len,
                );
                black_box((ptr, len));
            }
            upb_ffi::upb_bench_arena_free(arena);
        });
        let r_prost = run_bench(|| {
            for m in list {
                pvec.clear();
                m.encode(&mut pvec).unwrap();
                black_box(pvec.as_slice());
            }
        });
        print_row(
            label,
            "Serialize",
            r_roto.ns_per_op,
            r_upb_stack.ns_per_op,
            r_upb_heap.ns_per_op,
            r_prost.ns_per_op,
        );
    }
}
