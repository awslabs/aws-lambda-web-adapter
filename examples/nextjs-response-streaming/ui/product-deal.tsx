import { ProductCurrencySymbol } from '@/ui/product-currency-symbol';
import { toDecimal, type Dinero } from 'dinero.js';

export const ProductDeal = ({
    price: priceRaw,
    discount: discountRaw,
}: {
    price: Dinero<number>;
    discount: {
        amount: Dinero<number>;
    };
}) => {
    const discount = Number(toDecimal(discountRaw.amount));
    const price = Number(toDecimal(priceRaw));
    const percent = Math.round(100 - (discount / price) * 100);

    return (
        <div className="flex gap-x-1.5">
            <div className="text-lg font-bold leading-snug text-vercel-cyan">
                -{percent}%
            </div>
            <div className="flex">
                <div className="text-sm leading-snug text-white">
                    <ProductCurrencySymbol dinero={discountRaw.amount} />
                </div>
                <div className="text-lg font-bold leading-snug text-white">
                    {discount}
                </div>
            </div>
            <div className="text-sm leading-snug text-gray-400 line-through">
                <ProductCurrencySymbol dinero={priceRaw} />
                {price}
            </div>
        </div>
    );
};
