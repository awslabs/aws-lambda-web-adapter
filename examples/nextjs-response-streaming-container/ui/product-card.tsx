import { Product } from "@/app/api/products/product";
import { DineroSnapshot, dinero } from "dinero.js";
import Image from "next/image";
import Link from "next/link";
import { ProductBestSeller } from "./product-best-seller";
import { ProductEstimatedArrival } from "./product-estimated-arrival";
import { ProductLowStockWarning } from "./product-low-stock-warning";
import { ProductPrice } from "./product-price";
import { ProductRating } from "./product-rating";
import { ProductUsedPrice } from "./product-used-price";

export const ProductCard = ({
    product,
    href,
}: {
    product: Product;
    href: string;
}) => {
    const price = dinero(product.price as DineroSnapshot<number>);
    return (
        <Link href="" className="group block">
            <div className="space-y-2">
                <div className="relative">
                    {product.isBestSeller ? (
                        <div className="absolute top-2 left-2 z-10 flex">
                            <ProductBestSeller />
                        </div>
                    ) : null}
                    <Image
                        src={`${product.image}`}
                        width={400}
                        height={127}
                        className="rounded-xl grayscale group-hover:opacity-80"
                        alt={product.name}
                        placeholder="blur"
                        blurDataURL={product.imageBlur}
                    />
                </div>

                <div className="truncate text-sm font-medium text-white group-hover:text-vercel-cyan">
                    {product.name}
                </div>

                {product.rating ? <ProductRating rating={product.rating} /> : null}

                <ProductPrice price={price} discount={product.discount} />

                {product.usedPrice ? (
                    <ProductUsedPrice usedPrice={product.usedPrice} />
                ) : null}

                <ProductEstimatedArrival leadTime={product.leadTime} />

                {product.stock <= 1 ? (
                    <ProductLowStockWarning stock={product.stock} />
                ) : null}
            </div>
        </Link>
    )
}