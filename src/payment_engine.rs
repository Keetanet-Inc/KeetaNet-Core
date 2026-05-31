use std::fmt;
use regex::Regex;

// Константы клиринговых систем согласно CPA Rules (Payments Canada)
pub const CLR_SYS_CAD: &str = "ACS";
pub const CLR_SYS_USD: &str = "UBE";

// Регулярное выражение для Rule R14 (DPRN: 9 цифр, формат 0 + 3 банка + 5 филиала)
lazy_static::lazy_static! {
    static ref DPRN_RE: Regex = Regex::new(r"^0[0-9]{3}[0-9]{5}$").unwrap();
}

#[derive(Debug, Clone)]
pub enum KeetaPaymentError {
    InvalidMsgIdLength,       // Rule R3: > 35 символов
    SystemCurrencyMismatch,    // Rule R12: Несовпадение клиринговой системы и валюты
    InvalidRoutingNumber,      // Rule R14: Ошибка маски транзитного номера
}

impl fmt::Display for KeetaPaymentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeetaPaymentError::InvalidMsgIdLength => write!(f, "Reject (Rule R3): MsgId превышает 35 символов"),
            KeetaPaymentError::SystemCurrencyMismatch => write!(f, "Reject (Rule R12): Несоответствие клиринговой системы (ACS/UBE) и валюты (CAD/USD)"),
            KeetaPaymentError::InvalidRoutingNumber => write!(f, "Reject (Rule R14): Неверный формат маршрутного номера (DPRN)"),
        }
    }
}

/// Сердце заголовка межбанковского сообщения (Group Header ISO 20022)
pub struct KeetaGroupHeader {
    pub msg_id: String,       // Rule R3: max 35 символов (payment_intent.id)
    pub clr_system: String,   // Rule R1/R2: ACS или UBE
    pub cre_dt_tm: String,    // Время создания (ISO timestamp)
}

impl KeetaGroupHeader {
    pub fn new(msg_id: String, clr_system: String, cre_dt_tm: String) -> Self {
        Self { msg_id, clr_system, cre_dt_tm }
    }

    /// Проверка на соответствие правилам CPA Rules
    pub fn validate(&self, currency: &str) -> Result<(), KeetaPaymentError> {
        // 1. Валидация длины MsgId (Rule R3)
        if self.msg_id.len() > 35 {
            return Err(KeetaPaymentError::InvalidMsgIdLength);
        }

        // 2. Валидация взаимосвязи клиринга и валюты (Rule R12 / R1 / R2)
        match (self.clr_system.as_str(), currency) {
            (CLR_SYS_CAD, "CAD") => {}
            (CLR_SYS_USD, "USD") => {}
            _ => return Err(KeetaPaymentError::SystemCurrencyMismatch),
        }

        Ok(())
    }
}

/// Структура транзакции прямого дебета для pacs.003
pub struct AftDirectDebit {
    pub tx_id: String,
    pub amount: f64,          // Приводится из центов Stripe (дробное число)
    pub routing_number: String, // Маршрутный номер (DPRN) для Rule R14
    pub debtor_name: String,
}

impl AftDirectDebit {
    pub fn new(tx_id: String, stripe_cents: u64, routing_number: String, debtor_name: String) -> Self {
        Self {
            tx_id,
            amount: (stripe_cents as f64) / 100.0,
            routing_number,
            debtor_name,
        }
    }

    /// Проверка на легитимность маршрутного номера по канадским стандартам (Rule R14)
    pub fn validate_routing(&self) -> Result<(), KeetaPaymentError> {
        if !DPRN_RE.is_match(&self.routing_number) {
            return Err(KeetaPaymentError::InvalidRoutingNumber);
        }
        Ok(())
    }

    /// Легковесная потоковая сборка XML-документа pacs.003
    pub fn to_pacs003_xml(&self, header: &KeetaGroupHeader) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<Document xmlns="urn:iso:std:iso:20022:tech:xsd:pacs.003.001.06">
    <FIToFICstmrDrctDbt>
        <GrpHdr>
            <MsgId>{msg_id}</MsgId>
            <CreDtTm>{cre_dt_tm}</CreDtTm>
            <NbOfTxs>1</NbOfTxs>
            <SttlmInf>
                <SttlmMtd>CLRG</SttlmMtd>
                <ClrSys>
                    <Prtry>{clr_system}</Prtry>
                </ClrSys>
            </SttlmInf>
        </GrpHdr>
        <DrctDbtTxInf>
            <PmtId>
                <EndToEndId>{tx_id}</EndToEndId>
            </PmtId>
            <IntrbkSttlmAmt Ccy="{currency}">{amount:.2}</IntrbkSttlmAmt>
            <DbtrAgt>
                <FinInstnId>
                    <ClrSysMmbId>
                        <MmbId>{routing_number}</MmbId>
                    </ClrSysMmbId>
                </FinInstnId>
            </ DbtrAgt>
            <Dbtr>
                <Nm>{debtor_name}</Nm>
            </Dbtr>
        </DrctDbtTxInf>
    </FIToFICstmrDrctDbt>
</Document>"#,
            msg_id = header.msg_id,
            cre_dt_tm = header.cre_dt_tm,
            clr_system = header.clr_system,
            tx_id = self.tx_id,
            currency = if header.clr_system == CLR_SYS_CAD { "CAD" } else { "USD" },
            amount = self.amount,
            routing_number = self.routing_number,
            debtor_name = self.debtor_name
        )
    }
}