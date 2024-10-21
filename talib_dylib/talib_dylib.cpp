// talib_dylib.cpp : 定义 DLL 的导出函数。
//

#include "pch.h"
#include "framework.h"
#include "talib_dylib.h"
#include "include/ta_func.h"
#include <limits>

// 这是导出变量的一个示例
TALIBDYLIB_API int ntalibdylib = 0;

// 这是导出函数的一个示例。
TALIBDYLIB_API int fntalibdylib(void)
{
	return 0;
}

// 这是已导出类的构造函数。
Ctalibdylib::Ctalibdylib()
{
	return;
}

extern "C" {
	_declspec(dllexport) void sma(const double* inDouble, int inSize, int optInTimePeriod, double*outDouble) {
		int out_begin, out_size;
		int ret_code = TA_MA(0, inSize - 1, inDouble, optInTimePeriod, (TA_MAType)0, &out_begin, &out_size, outDouble);
		memmove(outDouble + out_begin, outDouble, out_size * sizeof(double));
		for (int i = 0;i < out_begin;++i) {
			outDouble[i] = std::numeric_limits<double>::quiet_NaN();
		}
	}
}