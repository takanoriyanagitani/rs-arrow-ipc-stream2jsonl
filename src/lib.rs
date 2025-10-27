#![forbid(clippy::unwrap_used)]
#![forbid(clippy::expect_used)]

use std::io;

use io::BufReader;
use io::Read;

use io::BufWriter;
use io::Write;

use arrow::datatypes::SchemaRef;
use arrow::record_batch::RecordBatch;

use arrow::ipc::reader::StreamReader;

use arrow::json::writer::LineDelimitedWriter;

pub struct IpcStreamReader<R>(pub StreamReader<BufReader<R>>);

impl<R: Read> IpcStreamReader<R> {
    pub fn schema_ref(&self) -> SchemaRef {
        self.0.schema()
    }

    pub fn into_batch_iter(self) -> impl Iterator<Item = Result<RecordBatch, io::Error>> {
        self.0.map(|r| r.map_err(io::Error::other))
    }
}

impl<R: Read> IpcStreamReader<R> {
    pub fn to_json_writer<W>(self, mut wtr: JsonLinesWriter<W>) -> Result<(), io::Error>
    where
        W: Write,
    {
        let values = self.into_batch_iter();
        wtr.write_all(values)?;
        wtr.finish()
    }

    pub fn to_writer<W>(self, mut wtr: W) -> Result<(), io::Error>
    where
        W: Write,
    {
        let jwtr = JsonLinesWriter(LineDelimitedWriter::new(&mut wtr));
        self.to_json_writer(jwtr)?;
        wtr.flush()
    }
}

impl<R: Read> IpcStreamReader<R> {
    pub fn from_reader(rdr: R, projection: Option<Vec<usize>>) -> Result<Self, io::Error> {
        let srdr: StreamReader<_> =
            StreamReader::try_new_buffered(rdr, projection).map_err(io::Error::other)?;
        Ok(Self(srdr))
    }
}

pub struct JsonLinesWriter<W: Write>(pub LineDelimitedWriter<W>);

impl<W: Write> JsonLinesWriter<W> {
    pub fn write_batch(&mut self, b: &RecordBatch) -> Result<(), io::Error> {
        self.0.write(b).map_err(io::Error::other)
    }

    pub fn finish(&mut self) -> Result<(), io::Error> {
        self.0.finish().map_err(io::Error::other)
    }
}

impl<W: Write> JsonLinesWriter<W> {
    pub fn write_all<I>(&mut self, b: I) -> Result<(), io::Error>
    where
        I: Iterator<Item = Result<RecordBatch, io::Error>>,
    {
        for rbat in b {
            let bat: RecordBatch = rbat?;
            self.write_batch(&bat)?;
        }
        Ok(())
    }
}

pub fn reader2ipc2jsons2writer<R, W>(
    rdr: R,
    wtr: W,
    projection: Option<Vec<usize>>,
) -> Result<(), io::Error>
where
    R: Read,
    W: Write,
{
    let irdr = IpcStreamReader::from_reader(rdr, projection)?;
    irdr.to_writer(wtr)
}

pub fn stdin2ipc2jsons2stdout(projection: Option<Vec<usize>>) -> Result<(), io::Error> {
    let i = io::stdin().lock();
    let mut o = io::stdout().lock();
    let bw = BufWriter::new(&mut o);
    reader2ipc2jsons2writer(i, bw, projection)?;
    o.flush()
}
