import { toSnapshot, type Dinero } from 'dinero.js';

export const ProductCurrencySymbol = ({
    dinero,
}: {
    dinero: Dinero<number>;
}) => {
    const snapshot = toSnapshot(dinero);
    let symbol = '';
    switch (snapshot.currency.code) {
        case 'GBP': {
            symbol = '£';
            break;
        }

        case 'EUR': {
            symbol = '€';
            break;
        }

        default: {
            symbol = '$';
            break;
        }
    }

    return <>{symbol}</>;
};
