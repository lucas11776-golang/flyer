use std::io::Result;

use async_std::task::block_on;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::join;
use tokio::net::{TcpListener, UdpSocket};
use hickory_proto::op::{Edns, Message, MessageType, Query, ResponseCode};
use hickory_proto::rr::{Name, RData, Record, RecordType, rdata::SOA};
use hickory_proto::serialize::binary::{BinDecodable, BinEncodable};
use url_domain_parse::utils::Domain;

const DNS_BUFFER_SIZE: usize = 1024;

pub fn run(domain_name: &str, host: &str, port: u16) {
    block_on(async {
        join!(udp(domain_name, host, port), tcp(domain_name, host, port));
    });
}

async fn udp(domain_name: &str, host: &str, port: u16) {
    let socket = UdpSocket::bind(format!("{}:{}", host, port)).await.unwrap();

    loop {
        let mut buf = [0u8; DNS_BUFFER_SIZE];
        let recv_result = socket.recv_from(&mut buf).await;

        if recv_result.is_err() {
            continue;
        }

        // TODO: add tokio::spawn
        let (size, peer_addr) = recv_result.unwrap();
        let query = Message::from_bytes(&buf[..size]).unwrap();
        let response = handle_query(domain_name, query).await.unwrap();
        let data = response.to_bytes().unwrap();
        let result = socket.send_to(&data, peer_addr).await;

        if result.is_err() {
            continue;
        }

        result.unwrap();
    }
}

async fn tcp(domain_name: &str, host: &str, port: u16) {
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await.unwrap();


    while let Ok((mut stream, _)) = listener.accept().await {
        let domain = String::from(domain_name);

        tokio::spawn(async move {
            let mut buf = [0u8; DNS_BUFFER_SIZE];
            let result = stream.read(&mut buf).await;

            if result.is_err() {
                return;
            }

            let size = result.unwrap();
            let query = Message::from_bytes(&buf[..size]).unwrap();
            let response = handle_query(&domain, query).await.unwrap();
            let data = response.to_bytes().unwrap();
            let result = stream.write(&data.to_bytes().unwrap()).await;

            if result.is_err() {
                return;
            }

            result.unwrap();
        });
    }
}

async fn handle_query(domain_name: &str, request: Message) -> Result<Message> {
    let mut response = Message::new();

    response.set_id(request.id());
    response.set_message_type(MessageType::Response);
    response.set_op_code(request.op_code());
    response.set_authoritative(true);
    response.set_recursion_desired(request.recursion_desired());
    response.set_recursion_available(request.recursion_desired());
    response.set_response_code(ResponseCode::NoError);

    for query in request.queries() {
        let domain = Domain::parse(query.name().to_string().trim_end_matches("."));

        if domain.host.is_none() {
            response.set_response_code(ResponseCode::BADNAME);

            return Ok(response);
        }

        let mut response_edns = Edns::new();

        response_edns.set_max_payload(DNS_BUFFER_SIZE.try_into().unwrap());
        response.set_edns(response_edns);
        response.add_query(query.clone());
        response.set_response_code(ResponseCode::NoError);

        let search_result = search_dns_record(domain_name, &mut response, query).await;

        if search_result.is_err() {
            response.set_response_code(ResponseCode::BADNAME);

            return Ok(response);
        }
    }

    Ok(response)
}

// TODO: check if domain match all host will be resolved :(
async fn search_dns_record(domain_name: &str, response: &mut Message, query: &Query) -> Result<()> {
    match query.query_type() {
        RecordType::A => {
            response.add_answer(Record::from_rdata(
                query.name().clone(),
                60,
                RData::A("127.0.0.1".parse().unwrap()),
            ));
        }
        RecordType::AAAA => {
            response.add_name_server(Record::from_rdata(
                query.name().clone(),
                60,
                RData::SOA(SOA::new(
                    // Name::from_str_relaxed("ns1.tracker.com.").unwrap(),
                    Name::from_str_relaxed(format!("ns1.{}.", domain_name)).unwrap(),
                    // Name::from_str_relaxed("admin.tracker.com.").unwrap(),
                    Name::from_str_relaxed(format!("admin.{}.", domain_name)).unwrap(),
                    2023101001,
                    3600,
                    600,
                    86400,
                    60,
                )),
            ));
        }
        RecordType::ANAME => todo!(),
        RecordType::ANY => todo!(),
        RecordType::AXFR => todo!(),
        RecordType::CAA => todo!(),
        RecordType::CDS => todo!(),
        RecordType::CDNSKEY => todo!(),
        RecordType::CERT => todo!(),
        RecordType::CNAME => todo!(),
        RecordType::CSYNC => todo!(),
        RecordType::DNSKEY => todo!(),
        RecordType::DS => todo!(),
        RecordType::HINFO => todo!(),
        RecordType::HTTPS => todo!(),
        RecordType::IXFR => todo!(),
        RecordType::KEY => todo!(),
        RecordType::MX => todo!(),
        RecordType::NAPTR => todo!(),
        RecordType::NS => todo!(),
        RecordType::NSEC => todo!(),
        RecordType::NSEC3 => todo!(),
        RecordType::NSEC3PARAM => todo!(),
        RecordType::NULL => todo!(),
        RecordType::OPENPGPKEY => todo!(),
        RecordType::OPT => todo!(),
        RecordType::PTR => todo!(),
        RecordType::RRSIG => todo!(),
        RecordType::SIG => todo!(),
        RecordType::SOA => todo!(),
        RecordType::SRV => todo!(),
        RecordType::SSHFP => todo!(),
        RecordType::SVCB => {

            //             // // HTTPS
//             // tracker.com. 3600 IN HTTPS 1 . (
//             //     alpn="http/1.1"
//             //     port="5354"
//             //     ipv4hint=127.0.0.1
//             // )

//             // // HTTP
//             // tracker.com. 3600 IN SVCB 1 . (
//             //     alpn="http/1.1"
//             //     port="5354"
//             //     ipv4hint=127.0.0.1
//             // )
            
//             // // HTTP
//             // tracker.com. 3600 IN A 127.0.0.1

//             // Only respond if querying tracker.com HTTPS/SVCB
//             // let name = Name::from_ascii("tracker.com.").unwrap();

//             // // TODO: fix
//             // use hickory_proto::rr::{Name, Record, RData};
//             // use hickory_proto::rr::rdata::svcb::{SVCB, SvcParamKey};
//             // use hickory_proto::rr::rdata::svcb::SvcParamValue::*;
//             // use std::net::Ipv4Addr;

//             // // Build SVCB RDATA
//             // let mut params = vec![
//             //     (SvcParamKek ::Port, Port(5354)),
//             //     (SvcParamKey::Ipv4Hint, Ipv4Hint(vec![Ipv4Addr::new(127, 0, 0, 1)])),
//             //     (SvcParamKey::Alpn, Alpn(vec![b"http/1.1".to_vec()])),
//             // ];

//             // let svcb = SVCB::new(
//             //     1,              // priority (ServiceMode)
//             //     Name::root(),   // "." = same name
//             //     params,
//             // );

//             // let record = Record::from_rdata(
//             //     name,
//             //     3600, // TTL
//             //     RData::SVCB(svcb),
//             // );

//             // response.add_answer(record);



            todo!()
        },
        RecordType::TLSA => todo!(),
        RecordType::TSIG => todo!(),
        RecordType::TXT => todo!(),
        RecordType::Unknown(_) => todo!(),
        RecordType::ZERO => todo!(),
        _ => {},
    };

    Ok(())
}