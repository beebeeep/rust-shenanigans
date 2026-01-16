use std::{thread::sleep, time::Duration};

use rdkafka::{
    ClientConfig, ClientContext, Message, TopicPartitionList,
    config::FromClientConfigAndContext,
    consumer::{BaseConsumer, Consumer, ConsumerContext, StreamConsumer},
    error::KafkaResult,
    producer::{BaseRecord, ProducerContext, ThreadedProducer},
};

struct DeliveryHandler {}

impl ClientContext for DeliveryHandler {}

impl ProducerContext for DeliveryHandler {
    type DeliveryOpaque = usize;

    fn delivery(
        &self,
        delivery_result: &rdkafka::message::DeliveryResult<'_>,
        delivery_opaque: Self::DeliveryOpaque,
    ) {
        match delivery_result.as_ref() {
            Ok(r) => {
                println!(
                    "delivery callback, offset {:?}, partition: {:?}, opaque: {:?}",
                    r.offset(),
                    r.partition(),
                    delivery_opaque
                );
            }
            Err(e) => {
                println!("delivery callback, delivery failed: {}", e.0)
            }
        }
    }
}

struct LoggingConsumerContext;

impl ClientContext for LoggingConsumerContext {}

impl ConsumerContext for LoggingConsumerContext {
    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        match result {
            Ok(_) => println!("Offsets committed successfully"),
            Err(e) => println!("Error while committing offsets: {}", e),
        };
    }
}

fn main() {
    let mut cfg = ClientConfig::new();
    cfg.set("bootstrap.servers", "localhost:9092");
    cfg.set("group.id", "chlos");
    cfg.set("enable.partition.eof", "false");
    cfg.set("enable.auto.commit", "false");
    cfg.set("enable.auto.offset.store", "false");
    let consumer = BaseConsumer::from_config_and_context(&cfg, LoggingConsumerContext {})
        .expect("creating consumer");
    consumer.subscribe(&["test-topic"]).expect("subscribing");
    for msg in &consumer {
        let msg = msg.expect("consuming message");
        let bmsg = msg.detach();
        let p = bmsg.payload().take().unwrap();

        println!(
            "got message partition {} offset {} key {:?} payload {}",
            msg.partition(),
            msg.offset(),
            msg.key(),
            String::from_utf8_lossy(msg.payload().unwrap()),
        );
        consumer
            .commit_message(&msg, rdkafka::consumer::CommitMode::Sync)
            .expect("committing offsets");
    }

    /*
    let producer = ThreadedProducer::from_config_and_context(&cfg, DeliveryHandler {})
        .expect("creating producer");
    producer
        .send(BaseRecord::<str, str, usize>::with_opaque_to("test-topic", 137).payload("CHLOS"))
        .expect("failed to enqueue");
    println!("done");
    sleep(Duration::from_millis(200));
    */
}
