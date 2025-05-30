﻿// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/* This file will be copied into the folder for generated code. */

namespace Azure.Iot.Operations.Protocol.UnitTests.Serializers.common
{
    using System;
    using System.Globalization;
    using System.Text.RegularExpressions;

    public class DecimalString
    {
        private static readonly Regex ValidationRegex = new Regex("^(?:\\+|-)?(?:[1-9][0-9]*|0)(?:\\.[0-9]*)?$", RegexOptions.Compiled);

        private readonly string value;

        public DecimalString()
            : this("0", skipValidation: false)
        {
        }

        public DecimalString(string value)
            : this(value, skipValidation: false)
        {
        }

        private DecimalString(string value, bool skipValidation)
        {
            if (!skipValidation && !ValidationRegex.IsMatch(value))
            {
                throw new ArgumentException($"string {value} is not a valid decimal value");
            }

            this.value = value;
        }

        public static implicit operator string(DecimalString decimalString) => decimalString.value;

        public static explicit operator DecimalString(string stringVal) => new DecimalString(stringVal);

        public static implicit operator double(DecimalString decimalString) => double.TryParse(decimalString.value, out double doubleVal) ? doubleVal : double.NaN;

        public static explicit operator DecimalString(double doubleVal) => new DecimalString(doubleVal.ToString("F", CultureInfo.InvariantCulture));

        public static bool operator !=(DecimalString? x, DecimalString? y)
        {
            if (ReferenceEquals(null, x))
            {
                return !ReferenceEquals(null, y);
            }

            return !x.Equals(y);
        }

        public static bool operator ==(DecimalString? x, DecimalString? y)
        {
            if (ReferenceEquals(null, x))
            {
                return ReferenceEquals(null, y);
            }

            return x.Equals(y);
        }

        public static bool TryParse(string value, out DecimalString? decimalString)
        {
            if (ValidationRegex.IsMatch(value))
            {
                decimalString = new DecimalString(value, skipValidation: true);
                return true;
            }
            else
            {
                decimalString = null;
                return false;
            }
        }

        public virtual bool Equals(DecimalString? other)
        {
            return other?.value == this?.value;
        }

        public override bool Equals(object? obj)
        {
            return obj is DecimalString other && Equals(other);
        }

        public override int GetHashCode()
        {
            return value.GetHashCode();
        }

        public override string ToString() => value;
    }
}
