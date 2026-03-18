use mml_def::{ImsUserNum, RangeOfPrefix, U4Number, U4NumberDivided, U4NumberVec};

fn main() {
    type ImsNum = U4Number<10>;
    //type NV = U4NumberVec<10>;
    let n1 = ImsNum::from(13360096745);
    let n2 = ImsNum::from("133600967450");
    let n3: ImsNum = "008675588998001".parse().unwrap();
    let n4 = ImsNum::from(&[b'1', b'4']);
    let n5 = ImsNum::from(1336009);
    println!("{} {} {} {}", n1, n2, n3, n4);
    println!("n2>n1 {},  n3>n1 {}  n4>n1 {}", n2 > n1, n3 > n1, n4 > n1);
    println!(
        "{} {} {} {}",
        n4.same_prefix_len(&ImsNum::from(1)),
        n4.same_prefix_len(&ImsNum::from(2)),
        n4.same_prefix_len(&ImsNum::from(14)),
        n4.same_prefix_len(&ImsNum::from(13))
    );
    println!(
        "{} {} {} {}",
        n5.same_prefix_len(&n1),
        n5.same_prefix_len(&ImsNum::from(133600)),
        n5.same_prefix_len(&ImsNum::from(1)),
        n5.same_prefix_len(&ImsNum::from("01"))
    );
    let rg1: RangeOfPrefix<12> = RangeOfPrefix::from_u4("82980000".into(), "82980098".into());
    rg1.into_iter().for_each(|n| print! {"{}; ", n});
    println!();
    let n10 = ImsNum::from(291389);
    let n11 = 2913900.into();
    println!("{} - {} ?Succ {}", n10, n11, n10.is_succeed_by(&n11));
    let n100 = ImsNum::from(230);
    println!("end=0 {}", n100.is_end_with(0));
    println!("len_prefix= {}", n100.same_prefix_len(&232.into()));
    let ns1: U4NumberVec<12> =
        "11,12,13,14,15,16,17,100,18, 19,2, 101,102,103,109,108,107,106,105,104"
            .parse()
            .unwrap();
    println!("{}", ns1);
    let nd = U4NumberDivided::new(ImsNum::from("12"), "12909".into());
    println!("{:?}", nd);
    let ns2 = U4NumberVec(nd.collect());
    println!("div= {}", ns2);
    type ImsUser = ImsUserNum<12>;
    let s = "debug";
    dbg!(s);
    let un1: ImsUser = "SIP:+8675588399211@gd.ctcims.cn"
        .parse::<ImsUser>()
        .unwrap();
    let un2: ImsUser = "tel:+8675526770123".parse().unwrap();
    let un3: ImsUser = "3.9.5.0.5.6.6.2.5.5.7.6.8.e164.arpa".parse().unwrap();
    let un4: ImsUser = "75588998001".parse().unwrap();
    let un5: ImsUser = "+866682775050@gd.ctcims.cn".parse().unwrap();
    println!("\"{}\"\t\"{}\"\t\"{}\"\t\"{}\"\t\"{}\"", un1, un2, un3, un4, un5);
    if let ImsUserNum::PUI(n) = un1 {
        let n = n.strip_prefix(&"755".into()).copy_prefix(7);
        //let n = n.strip_prefix(&"755".into());
        let n = n.with_prefix(&755.into());
        println!("PFX = {}\n", n);
        let (mut n, c) = n.to_snb(11).unwrap();
        for i in 0..c {
            print!("{};   ", ImsUserNum::PUI(n));
            if (i + 1) % 5 == 0 {
                println!();
            }
            n = n.get_succeed_num().unwrap();
        }
    }
}
