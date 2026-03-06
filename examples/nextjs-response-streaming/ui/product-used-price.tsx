import { Product } from '@/app/api/products/product';
import { dinero, toDecimal, type DineroSnapshot } from 'dinero.js';

export const ProductUsedPrice = ({
    usedPrice: usedPriceRaw,
}: {
    usedPrice: Product['usedPrice'];
}) => {
    const usedPrice = dinero(usedPriceRaw as DineroSnapshot<number>);

    return (
        <div className="text-sm">
            <div className="text-gray-400">More buying choices</div>
            <div className="text-gray-200">
                ${Math.ceil(Number(toDecimal(usedPrice)))} (used)
            </div>
        </div>
    );
};
