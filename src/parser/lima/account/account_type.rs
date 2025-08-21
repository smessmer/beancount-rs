use crate::model::AccountType;

impl From<beancount_parser_lima::AccountType> for AccountType {
    fn from(value: beancount_parser_lima::AccountType) -> Self {
        match value {
            beancount_parser_lima::AccountType::Assets => AccountType::Assets,
            beancount_parser_lima::AccountType::Liabilities => AccountType::Liabilities,
            beancount_parser_lima::AccountType::Equity => AccountType::Equity,
            beancount_parser_lima::AccountType::Income => AccountType::Income,
            beancount_parser_lima::AccountType::Expenses => AccountType::Expenses,
        }
    }
}
