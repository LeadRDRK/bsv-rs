use std::fs::File;

use bsv::reader::BsvReader;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the BSV file
    path: String,

    /// Dump file information and data (default)
    #[arg(short, long)]
    dump: bool,

    /// Output in CSV format
    #[arg(short, long)]
    csv: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut bsv = BsvReader::new(File::open(args.path)?)?;

    if args.csv {
        let mut writer = csv::Writer::from_writer(std::io::stdout());
        while let Some(row) = bsv.next()? {
            for column in row {
                if let Some(value) = column.unum() {
                    writer.write_field(value.to_string())?;
                }
                else if let Some(value) = column.blob() {
                    writer.write_field(value.into_iter()
                        .map(|v| v.to_string())
                        .collect::<Vec<_>>()
                        .join(",")
                    )?;
                }
                else if let Some(value) = column.text() {
                    writer.write_field(value)?;
                }
                else {
                    writer.write_field("[Unknown]")?;
                }
            }
            writer.write_record(std::iter::empty::<&[u8]>())?;
        }
    }
    else {
        let header = bsv.header();
        println!(
            "-- Properties\n\
            Row count: {}\n\
            Max row size: {}\n\
            Schema version: {}\n",
            header.row_count(), header.max_row_size(), header.schema_version()
        );

        println!("-- Column schemas");
        println!("[{}]\n", header.schemas()
            .iter()
            .map(|schema| match schema.fixed_size {
                Some(size) => format!("{:?}({})", schema.value_type, size),
                None => format!("{:?}", schema.value_type),
            })
            .collect::<Vec<_>>()
            .join(", ")
        );

        println!("-- Data");
        while let Some(row) = bsv.next()? {
            println!("{:?}", row);
        }
    }

    Ok(())
}
